use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context, Line},
        Widget,
    },
};

// --- 画布与布局配置 ---
const CANVAS_X_BOUND: f64 = 80.0;
const CANVAS_Y_BOUND: f64 = 40.0;
// 眼睛配置
const EYE_X_OFFSET: f64 = 18.0;
const EYE_DEFAULT_WIDTH: f64 = 14.0;
const EYE_DEFAULT_HEIGHT: f64 = 16.0;

// --- 🎨 赛博霓虹配色 (高亮 RGB) ---
// 这里的颜色特意调高了亮度，配合黑色背景会有"荧光"感
const COLOR_IDLE: Color = Color::Rgb(0, 245, 255);      // 赛博蓝 (Cyber Cyan)
const COLOR_LISTENING: Color = Color::Rgb(57, 255, 20); // 荧光绿 (Neon Green) - 极其明亮
const COLOR_SPEAKING: Color = Color::Rgb(255, 40, 220); // 霓虹紫 (Neon Magenta)
const COLOR_THINKING: Color = Color::Rgb(255, 215, 0);  // 琥珀金 (Amber Gold)
const COLOR_DIM: Color = Color::Rgb(60, 60, 80);        // 暗色装饰

/// 表情状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceState {
    Idle,      // 待机：呼吸，偶尔眨眼
    Listening, // 聆听：眼睛瞪大，颜色变亮
    Speaking,  // 说话：嘴巴律动
    Thinking,  // 思考：眼睛眯起来，有粒子动效
}

/// 动画状态机（包含物理属性，用于平滑过渡）
pub struct FaceAnimator {
    state: FaceState,
    frame: u64,
    
    // 物理属性 (用于平滑插值)
    current_eye_height: f64,
    current_eye_width: f64,
    
    // 瞳孔/注视点偏移 (x, y)
    look_offset: (f64, f64),
    target_look_offset: (f64, f64),
    
    // 眨眼逻辑
    next_blink_frame: u64,
    is_blinking: bool,
    
    // 粒子系统
    particles: Vec<Particle>,
}

struct Particle {
    x: f64,
    y: f64,
    speed: f64,
    size: f64,
    color_offset: u8,
}

impl FaceAnimator {
    pub fn new() -> Self {
        Self {
            state: FaceState::Idle,
            frame: 0,
            current_eye_height: EYE_DEFAULT_HEIGHT,
            current_eye_width: EYE_DEFAULT_WIDTH,
            look_offset: (0.0, 0.0),
            target_look_offset: (0.0, 0.0),
            next_blink_frame: 60,
            is_blinking: false,
            particles: Vec::new(),
        }
    }

    pub fn set_state(&mut self, state: FaceState) {
        if self.state != state {
            self.state = state;
            self.is_blinking = false;
            
            // 状态切换时的初始化
            match state {
                FaceState::Listening => {
                    self.target_look_offset = (0.0, 0.0);
                    // 切换到聆听时，眼睛瞬间睁大一点点，增加灵动感
                    self.current_eye_height = 2.0; 
                },
                FaceState::Thinking => self.particles.clear(),
                _ => {}
            }
        }
    }

    pub fn state(&self) -> FaceState {
        self.state
    }

    pub fn tick(&mut self) {
        self.frame += 1;

        // 1. 眨眼逻辑
        if !self.is_blinking && self.frame >= self.next_blink_frame {
            self.is_blinking = true;
        }

        // 2. 计算眼睛目标尺寸
        let mut target_h = EYE_DEFAULT_HEIGHT;
        let mut target_w = EYE_DEFAULT_WIDTH;

        match self.state {
            FaceState::Idle => {
                // 呼吸效果：让眼睛稍微缩放
                let breath = (self.frame as f64 * 0.08).sin() * 0.8;
                target_h += breath;
                target_w += breath * 0.6;
                
                // 随机注视
                if self.frame % 120 == 0 {
                    let mut rng = rand::thread_rng();
                    // 稍微平滑一点的随机注视
                    if rng.gen_bool(0.7) {
                        self.target_look_offset = (
                            rng.gen_range(-4.0..4.0),
                            rng.gen_range(-2.0..2.0)
                        );
                    } else {
                        self.target_look_offset = (0.0, 0.0);
                    }
                }
            }
            FaceState::Listening => {
                // 聆听：大圆眼
                target_h = 18.0;
                target_w = 18.0;
                self.target_look_offset = (0.0, 0.0);
            }
            FaceState::Thinking => {
                // 思考：眯眼
                target_h = 3.5; 
                target_w = 14.0;
                // 向上看
                self.target_look_offset = (0.0, 6.0);
                self.update_particles();
            }
            FaceState::Speaking => {
                // 说话：稍微扁一点
                target_h = 10.0;
                target_w = 15.0;
                self.target_look_offset = (0.0, 0.0);
            }
        }

        // 眨眼处理
        if self.is_blinking {
            target_h = 0.5; // 闭眼
            target_w = 16.0; // 闭眼时稍微变宽
            
            if self.frame >= self.next_blink_frame + 5 {
                self.is_blinking = false;
                let mut rng = rand::thread_rng();
                self.next_blink_frame = self.frame + rng.gen_range(80..200);
            }
        }

        // 3. 物理插值 (Lerp) - 增加 smooth_factor 让动画更跟手
        let smooth_factor = 0.3;
        self.current_eye_height += (target_h - self.current_eye_height) * smooth_factor;
        self.current_eye_width += (target_w - self.current_eye_width) * smooth_factor;
        
        let look_smooth = 0.1;
        self.look_offset.0 += (self.target_look_offset.0 - self.look_offset.0) * look_smooth;
        self.look_offset.1 += (self.target_look_offset.1 - self.look_offset.1) * look_smooth;
    }

    fn update_particles(&mut self) {
        let mut rng = rand::thread_rng();
        if self.particles.len() < 8 && rng.gen_bool(0.15) {
            self.particles.push(Particle {
                x: rng.gen_range(-8.0..8.0),
                y: -12.0, // 从嘴巴附近生成
                speed: rng.gen_range(0.3..0.7),
                size: rng.gen_range(1.0..2.5),
                color_offset: rng.r#gen(),
            });
        }

        for p in &mut self.particles {
            p.y += p.speed;
            p.x += (self.frame as f64 * 0.15 + p.y).sin() * 0.3; // 螺旋上升
        }
        self.particles.retain(|p| p.y < 25.0);
    }

    pub fn widget(&self) -> FaceWidget {
        FaceWidget { animator: self }
    }
}

pub struct FaceWidget<'a> {
    animator: &'a FaceAnimator,
}

impl Widget for FaceWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.animator.state;
        let frame = self.animator.frame;
        
        let main_color = match state {
            FaceState::Idle => COLOR_IDLE,
            FaceState::Listening => COLOR_LISTENING,
            FaceState::Speaking => COLOR_SPEAKING,
            FaceState::Thinking => COLOR_THINKING,
        };

        Canvas::default()
            .block(ratatui::widgets::Block::default())
            .marker(Marker::Braille)
            .x_bounds([-CANVAS_X_BOUND / 2.0, CANVAS_X_BOUND / 2.0])
            .y_bounds([-CANVAS_Y_BOUND / 2.0, CANVAS_Y_BOUND / 2.0])
            .paint(|ctx| {
                // 1. 绘制眼睛
                let eye_w = self.animator.current_eye_width;
                let eye_h = self.animator.current_eye_height;
                let (look_x, look_y) = self.animator.look_offset;

                // 为了让线条看起来更"实"、更亮，我们画两层
                // 外层：主轮廓
                draw_eye_pair(ctx, look_x, look_y, eye_w, eye_h, main_color);
                
                // 内层：稍微缩小一点，增加厚度感 (Pseudo-bold)
                // 只有当眼睛张开比较大时才画内圈，避免眯眼时糊在一起
                if eye_h > 4.0 {
                     draw_eye_pair(ctx, look_x, look_y, eye_w * 0.85, eye_h * 0.85, main_color);
                }

                // 2. 绘制嘴巴 / 状态特效
                match state {
                    FaceState::Speaking => {
                        // 频谱式声波嘴巴
                        let width = 24.0;
                        let segments = 24;
                        for i in 0..segments {
                            let x_norm = i as f64 / segments as f64;
                            let x = (x_norm - 0.5) * width;
                            
                            // 模拟对称声波
                            let dist_from_center = (x_norm - 0.5).abs();
                            let envelope = 1.0 - dist_from_center * 2.0; // 中间高两边低
                            
                            let phase = frame as f64 * 0.5 + i as f64 * 0.5;
                            let amp = 5.0 * envelope + (phase.sin() * 3.0 * envelope);
                            let y_base = -12.0;
                            
                            ctx.draw(&Line {
                                x1: x, y1: y_base - amp,
                                x2: x, y2: y_base + amp,
                                color: main_color,
                            });
                        }
                    }
                    FaceState::Thinking => {
                        // 粒子泡泡
                        for p in &self.animator.particles {
                            draw_circle(ctx, p.x, -5.0 + p.y, p.size, main_color);
                        }
                        // 嘴巴是一个小圆点
                        draw_circle(ctx, 0.0, -12.0, 1.5, main_color);
                        draw_circle(ctx, 0.0, -12.0, 0.5, Color::White); // 增加高光
                    }
                    FaceState::Listening => {
                        // 张开的嘴巴，画两层增加亮度
                        draw_ellipse(ctx, 0.0, -14.0, 4.0, 3.0, main_color);
                        draw_ellipse(ctx, 0.0, -14.0, 3.0, 2.0, main_color);
                    }
                    FaceState::Idle => {
                        // 微笑弧线
                        // 使用多个短线段拟合平滑曲线
                        let smile_w = 8.0;
                        let smile_h = 2.5;
                        let steps = 10;
                        for i in 0..steps {
                            let t1 = i as f64 / steps as f64;
                            let t2 = (i + 1) as f64 / steps as f64;
                            
                            let x1 = (t1 - 0.5) * smile_w;
                            let y1 = -13.0 + (t1 - 0.5).powi(2) * smile_h;
                            
                            let x2 = (t2 - 0.5) * smile_w;
                            let y2 = -13.0 + (t2 - 0.5).powi(2) * smile_h;
                            
                            ctx.draw(&Line { x1, y1, x2, y2, color: COLOR_DIM }); // 暗一点
                        }
                    }
                }
            })
            .render(area, buf);
    }
}

// --- 辅助绘图函数 ---

fn draw_eye_pair(ctx: &mut Context, off_x: f64, off_y: f64, w: f64, h: f64, color: Color) {
    // 左眼
    draw_ellipse(ctx, -EYE_X_OFFSET + off_x, 6.0 + off_y, w, h, color);
    // 右眼
    draw_ellipse(ctx, EYE_X_OFFSET + off_x, 6.0 + off_y, w, h, color);
}

// 通用椭圆绘制 (通过32边形拟合)
fn draw_ellipse(ctx: &mut Context, cx: f64, cy: f64, rx: f64, ry: f64, color: Color) {
    let segments = 32; // 增加段数让圆形更平滑
    let mut points = Vec::with_capacity(segments + 1);
    
    for i in 0..=segments {
        let theta = (i as f64 / segments as f64) * std::f64::consts::PI * 2.0;
        let x = cx + rx * theta.cos();
        let y = cy + ry * theta.sin();
        points.push((x, y));
    }

    for i in 0..segments {
        ctx.draw(&Line {
            x1: points[i].0,
            y1: points[i].1,
            x2: points[i+1].0,
            y2: points[i+1].1,
            color,
        });
    }
    
    // 如果高度很小（比如眨眼），强制画一条水平线保证可见性
    if ry < 1.0 {
         ctx.draw(&Line {
            x1: cx - rx, y1: cy,
            x2: cx + rx, y2: cy,
            color,
        });
    }
}

fn draw_circle(ctx: &mut Context, cx: f64, cy: f64, r: f64, color: Color) {
    draw_ellipse(ctx, cx, cy, r, r, color);
}