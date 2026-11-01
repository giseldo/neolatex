use crossbeam_channel::{bounded, Receiver, Sender};
use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Debug)]
pub enum Msg {
    Ok  { pdf: PathBuf, log: String },
    Err { log: String },
}

pub struct Compiler {
    tx:      Sender<(String, PathBuf)>,
    result:  Arc<Mutex<Option<Msg>>>,
    running: Arc<Mutex<bool>>,
}

impl Compiler {
    pub fn new() -> Self {
        let (tx, rx): (Sender<(String, PathBuf)>, Receiver<(String, PathBuf)>) = bounded(1);
        let result  = Arc::new(Mutex::new(None));
        let running = Arc::new(Mutex::new(false));

        let (result2, running2) = (result.clone(), running.clone());

        thread::spawn(move || {
            while let Ok((src, dir)) = rx.recv() {
                *running2.lock().unwrap() = true;

                let tex_path = dir.join("main.tex");

                let msg = match std::fs::write(&tex_path, &src) {
                    Err(e) => Msg::Err { log: format!("Cannot write .tex: {e}") },
                    Ok(_)  => run_pdflatex(&dir),
                };

                *result2.lock().unwrap() = Some(msg);
                *running2.lock().unwrap() = false;
            }
        });

        Self { tx, result, running }
    }

    pub fn compile(&self, src: &str, dir: &PathBuf) {
        let _ = self.tx.try_send((src.to_string(), dir.clone()));
    }

    pub fn is_running(&self) -> bool { *self.running.lock().unwrap() }
    pub fn take(&self)      -> Option<Msg> { self.result.lock().unwrap().take() }
}

fn run_pdflatex(dir: &PathBuf) -> Msg {
    let mut cmd = Command::new("pdflatex");
    cmd.current_dir(dir)
        .args(["-interaction=nonstopmode", "-file-line-error", "main.tex"])
        .stdin(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.output() {
        Ok(out) => {
            let log = String::from_utf8_lossy(&out.stdout).into_owned();
            let pdf = dir.join("main.pdf");
            if pdf.exists() {
                Msg::Ok { pdf, log }
            } else {
                Msg::Err { log: format!("Compilation failed.\n{log}") }
            }
        }
        Err(e) => Msg::Err {
            log: format!(
                "pdflatex not found: {e}\n\n\
                 Make sure TeX Live or MiKTeX is installed and pdflatex is in PATH."
            ),
        },
    }
}
