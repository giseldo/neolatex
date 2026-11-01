use crate::{
    compiler::{Compiler, Msg},
    editor,
    preview::PdfPreview,
    theme,
};
use egui::{
    Align, Button, CentralPanel, Color32, Frame, Key, Layout, Margin, Response, RichText,
    Rounding, SidePanel, Stroke, TopBottomPanel, Ui,
};
use std::{path::PathBuf, time::{Duration, Instant}};

const TEMPLATE: &str = r"\documentclass{article}
\usepackage[utf8]{inputenc}
\usepackage{amsmath, amssymb}

\title{Meu Documento}
\author{Autor}
\date{\today}

\begin{document}

\maketitle

\section{Introdução}

Bem-vindo ao \textbf{NeoLatex} — um editor \LaTeX{} rápido e elegante.

\begin{equation}
  \int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}
\end{equation}

\section{Lista}

\begin{itemize}
  \item Primeiro item
  \item Segundo item com $\alpha + \beta = \gamma$
\end{itemize}

\end{document}
";

const DEFAULT_FONT: f32 = 14.0;

#[derive(PartialEq, Clone, Copy)]
enum State { Idle, Running, Ok, Err }

pub struct NeoLatex {
    content:   String,
    file_path: Option<PathBuf>,
    modified:  bool,
    font_size: f32,
    cursor:    (usize, usize),

    compiler:  Compiler,
    work_dir:  tempfile::TempDir,
    state:     State,
    started:   Option<Instant>,
    last_ms:   Option<u128>,
    log:       String,
    show_log:  bool,

    preview:   PdfPreview,
    auto:      bool,
    last_edit: Option<Instant>,
    title:     String,
}

impl NeoLatex {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            content:   TEMPLATE.to_string(),
            file_path: None,
            modified:  false,
            font_size: DEFAULT_FONT,
            cursor:    (1, 1),
            compiler:  Compiler::new(),
            work_dir:  tempfile::tempdir().expect("temp dir"),
            state:     State::Idle,
            started:   None,
            last_ms:   None,
            log:       String::new(),
            show_log:  false,
            preview:   PdfPreview::new(),
            auto:      true,
            last_edit: None,
            title:     String::new(),
        };
        app.compile();
        app
    }

    fn compile(&mut self) {
        if self.compiler.is_running() { return; }
        self.started = Some(Instant::now());
        self.state = State::Running;
        self.compiler.compile(&self.content, &self.work_dir.path().to_path_buf());
    }

    fn save(&mut self) {
        match self.file_path.clone() {
            Some(p) => {
                if std::fs::write(&p, &self.content).is_ok() { self.modified = false; }
            }
            None => self.save_as(),
        }
    }

    fn save_as(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .set_title("Salvar arquivo LaTeX")
            .add_filter("LaTeX", &["tex"])
            .save_file()
        {
            if std::fs::write(&p, &self.content).is_ok() {
                self.file_path = Some(p);
                self.modified = false;
            }
        }
    }

    fn open(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .set_title("Abrir arquivo LaTeX")
            .add_filter("LaTeX", &["tex"])
            .pick_file()
        {
            self.load(p);
        }
    }

    fn load(&mut self, p: PathBuf) {
        if let Ok(txt) = std::fs::read_to_string(&p) {
            self.content = txt;
            self.file_path = Some(p);
            self.modified = false;
            self.compile();
        }
    }

    fn new_file(&mut self) {
        self.content = TEMPLATE.to_string();
        self.file_path = None;
        self.modified = false;
        self.compile();
    }

    fn file_name(&self) -> String {
        self.file_path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sem título.tex".into())
    }

    // ── background results, debounce, shortcuts ──────────────────────────────
    fn tick(&mut self, ctx: &egui::Context) {
        if let Some(msg) = self.compiler.take() {
            self.last_ms = self.started.map(|t| t.elapsed().as_millis());
            match msg {
                Msg::Ok { pdf, log } => {
                    self.preview.reload(&pdf, ctx);
                    self.log = log;
                    self.state = State::Ok;
                }
                Msg::Err { log } => {
                    self.log = log;
                    self.state = State::Err;
                    self.show_log = true;
                }
            }
        }
        if self.compiler.is_running() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        if self.auto {
            if let Some(t) = self.last_edit {
                if t.elapsed() > Duration::from_millis(900) {
                    self.last_edit = None;
                    self.compile();
                } else {
                    ctx.request_repaint_after(Duration::from_millis(150));
                }
            }
        }

        // Read keys first, act afterwards: file dialogs must not run inside the input lock.
        let (f5, save, open, new, zin, zout, zreset, dropped) = ctx.input(|i| {
            let c = i.modifiers.command;
            (
                i.key_pressed(Key::F5),
                c && i.key_pressed(Key::S),
                c && i.key_pressed(Key::O),
                c && i.key_pressed(Key::N),
                c && i.key_pressed(Key::Equals),
                c && i.key_pressed(Key::Minus),
                c && i.key_pressed(Key::Num0),
                i.raw.dropped_files.first().and_then(|f| f.path.clone()),
            )
        });
        if f5 { self.compile(); }
        if save { self.save(); self.compile(); }
        if open { self.open(); }
        if new { self.new_file(); }
        if zin { self.font_size = (self.font_size + 1.0).min(28.0); }
        if zout { self.font_size = (self.font_size - 1.0).max(9.0); }
        if zreset { self.font_size = DEFAULT_FONT; }
        if let Some(p) = dropped { self.load(p); }

        let title = format!("{}{} — NeoLatex", self.file_name(), if self.modified { " ●" } else { "" });
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    // ── UI sections ──────────────────────────────────────────────────────────
    fn toolbar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("toolbar")
            .frame(Frame::none()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .inner_margin(Margin::symmetric(12.0, 7.0)))
            .show(ctx, |ui| {
                // ghost buttons: no fill until hovered
                let w = &mut ui.style_mut().visuals.widgets;
                w.inactive.weak_bg_fill = Color32::TRANSPARENT;
                w.inactive.bg_fill = Color32::TRANSPARENT;

                ui.horizontal(|ui| {
                    ui.label(RichText::new("NeoLatex").size(15.0).strong().color(theme::ACCENT));
                    ui.add_space(6.0);

                    if flat(ui, "Novo").on_hover_text("Ctrl+N").clicked()   { self.new_file(); }
                    if flat(ui, "Abrir").on_hover_text("Ctrl+O").clicked()  { self.open(); }
                    if flat(ui, "Salvar").on_hover_text("Ctrl+S").clicked() { self.save(); }

                    ui.add_space(4.0);
                    let busy = self.state == State::Running;
                    let compile = Button::new(
                        RichText::new(if busy { "Compilando…" } else { "▶  Compilar" })
                            .strong().color(Color32::WHITE),
                    )
                    .fill(if busy { theme::ACCENT.gamma_multiply(0.45) } else { theme::ACCENT })
                    .rounding(Rounding::same(6.0));
                    if ui.add_enabled(!busy, compile).on_hover_text("F5").clicked() {
                        self.compile();
                    }

                    ui.add_space(4.0);
                    ui.checkbox(&mut self.auto, RichText::new("Auto").color(theme::TEXT))
                        .on_hover_text("Recompila automaticamente ao parar de digitar");
                    if ui.selectable_label(self.show_log, RichText::new("Log").color(theme::TEXT)).clicked() {
                        self.show_log = !self.show_log;
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let name = self.file_name();
                        ui.label(RichText::new(name).size(13.0).color(theme::TEXT));
                        if self.modified {
                            dot(ui, theme::WARN);
                        }
                    });
                });
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("status")
            .exact_height(26.0)
            .frame(Frame::none()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .inner_margin(Margin::symmetric(12.0, 0.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let small = |t: String| RichText::new(t).size(12.0).color(theme::MUTED);

                    let (color, text) = match self.state {
                        State::Idle    => (theme::MUTED, "Pronto".to_string()),
                        State::Running => (theme::ACCENT, "Compilando…".to_string()),
                        State::Ok      => (theme::OK, match self.last_ms {
                            Some(ms) if ms >= 1000 => format!("Compilado em {:.1} s", ms as f32 / 1000.0),
                            Some(ms) => format!("Compilado em {ms} ms"),
                            None => "Compilado".into(),
                        }),
                        State::Err     => (theme::ERR, "Erro de compilação".to_string()),
                    };
                    dot(ui, color);
                    ui.label(RichText::new(text).size(12.0).color(theme::TEXT));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(small("LaTeX".into()));
                        ui.label(small("UTF-8".into()));
                        ui.label(small(format!("Fonte {}", self.font_size as u32)));
                        ui.label(small(format!("{} linhas", self.content.bytes().filter(|&c| c == b'\n').count() + 1)));
                        ui.label(small(format!("Ln {}, Col {}", self.cursor.0, self.cursor.1)));
                        if self.preview.page_count() > 0 {
                            let n = self.preview.page_count();
                            ui.label(small(format!("{n} {}", if n == 1 { "página" } else { "páginas" })));
                        }
                    });
                });
            });
    }

    fn log_panel(&mut self, ctx: &egui::Context) {
        if !self.show_log { return; }
        TopBottomPanel::bottom("log")
            .resizable(true)
            .min_height(80.0)
            .default_height(170.0)
            .frame(Frame::none()
                .fill(theme::BG)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .inner_margin(Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("LOG DE COMPILAÇÃO").size(11.0).strong().color(theme::MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("✕").clicked() { self.show_log = false; }
                    });
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for line in self.log.lines() {
                        let color = if line.starts_with('!') || line.contains(".tex:") || line.contains("Error") {
                            theme::ERR
                        } else if line.contains("Warning") || line.contains("Overfull") || line.contains("Underfull") {
                            theme::WARN
                        } else {
                            theme::MUTED
                        };
                        ui.label(RichText::new(line).monospace().size(11.5).color(color));
                    }
                });
            });
    }
}

impl eframe::App for NeoLatex {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);

        self.toolbar(ctx);
        self.status_bar(ctx);
        self.log_panel(ctx);

        SidePanel::left("editor")
            .resizable(true)
            .default_width(ctx.screen_rect().width() * 0.5)
            .min_width(320.0)
            .frame(Frame::none().fill(theme::BG))
            .show(ctx, |ui| {
                let out = editor::show(ui, &mut self.content, self.font_size);
                self.cursor = (out.line, out.col);
                if out.changed {
                    self.modified = true;
                    if self.auto { self.last_edit = Some(Instant::now()); }
                }
            });

        CentralPanel::default()
            .frame(Frame::none().fill(theme::PREVIEW_BG))
            .show(ctx, |ui| self.preview.ui(ui));
    }
}

fn flat(ui: &mut Ui, text: &str) -> Response {
    ui.add(Button::new(RichText::new(text).size(13.0).color(theme::TEXT)))
}

fn dot(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}
