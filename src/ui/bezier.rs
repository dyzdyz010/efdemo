use egui::*;
use petgraph::graph::EdgeIndex;

use crate::canvas::CanvasState;

#[derive(Debug)]
pub struct Anchor {
    pos: egui::Pos2,        // 锚点坐标
    handle_in: egui::Pos2,  // 进入方向控制柄
    handle_out: egui::Pos2, // 退出方向控制柄
    is_smooth: bool,        // 是否平滑锚点（控制柄对称）
    selected: bool,         // 是否被选中
}

impl Anchor {
    // 创建平滑锚点（自动生成对称控制柄）
    pub fn new_smooth(pos: egui::Pos2) -> Self {
        let handle_offset = Vec2::new(30.0, 0.0); // 默认水平对称
        Self {
            pos,
            handle_in: pos - handle_offset,
            handle_out: pos + handle_offset,
            is_smooth: true,
            selected: false,
        }
    }

    // 创建带自定义控制柄的锚点（默认非平滑）
    pub fn with_handles(pos: egui::Pos2, handle_in: egui::Pos2, handle_out: egui::Pos2) -> Self {
        Self {
            pos,
            handle_in,
            handle_out,
            is_smooth: false, // 需要手动调用 set_smooth(true) 启用平滑
            selected: false,
        }
    }

    // 添加设置平滑状态的方法
    pub fn set_smooth(&mut self, is_smooth: bool) {
        self.is_smooth = is_smooth;
        if is_smooth {
            self.enforce_smooth();
        }
    }

    // 强制保持控制柄对称
    pub fn enforce_smooth(&mut self) {
        let delta_in = self.handle_in - self.pos;
        let delta_out = self.handle_out - self.pos;

        // 如果两个控制柄都非零，取平均值
        if delta_in != Vec2::ZERO && delta_out != Vec2::ZERO {
            let avg_dir = (delta_in.normalized() - delta_out.normalized()) / 2.0;
            self.handle_in = self.pos + avg_dir * delta_in.length();
            self.handle_out = self.pos - avg_dir * delta_out.length();
        }
        // 否则保持反向
        else {
            self.handle_out = self.pos - delta_in;
            self.handle_in = self.pos - delta_out;
        }
    }
}

pub struct BezierWidget<'a> {
    pub canvas_state: &'a mut CanvasState,
    pub edge_index: EdgeIndex,

    pub anchors: Vec<Anchor>,       // 所有锚点
    pub dragging: Option<DragType>, // 当前拖动的对象类型（锚点、控制柄）
}

#[derive(Debug)]
pub enum DragType {
    Anchor(usize),    // 拖动的锚点索引
    HandleIn(usize),  // 拖动的进入控制柄索引
    HandleOut(usize), // 拖动的退出控制柄索引
}

impl<'a> Widget for BezierWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        println!(
            "anchors: {:?}",
            self.anchors.iter().map(|a| a.pos).collect::<Vec<_>>()
        );
        let (pos, desired_size) = self.desired_size();
        let rect = Rect::from_min_size(pos, desired_size);
        let screen_rect = self.canvas_state.to_screen_rect(rect);

        let response = ui.allocate_rect(screen_rect, Sense::click_and_drag());
        self.draw_bezier(ui.painter());
        self.apply_actions(&response, ui);
        response
    }
}

impl<'a> BezierWidget<'a> {
    pub fn new(
        anchors: Vec<Anchor>,
        canvas_state: &'a mut CanvasState,
        edge_index: EdgeIndex,
    ) -> Self {
        Self {
            anchors,
            canvas_state,
            edge_index,
            dragging: None,
        }
    }

    fn desired_size(&self) -> (Pos2, Vec2) {
        // 用控制点计算边界，能够将所有控制点都包含在内的外接矩形
        let mut min = self.anchors[0].pos;
        let mut max = self.anchors[0].pos;
        for anchor in &self.anchors {
            min.x = min.x.min(anchor.pos.x);
            min.y = min.y.min(anchor.pos.y);
            max.x = max.x.max(anchor.pos.x);
            max.y = max.y.max(anchor.pos.y);
        }
        // let min = self
        //     .anchors
        //     .iter()
        //     .min_by(|a, b| {
        //         a.pos
        //             .x
        //             .partial_cmp(&b.pos.x)
        //             .unwrap()
        //             .then(a.pos.y.partial_cmp(&b.pos.y).unwrap())
        //     })
        //     .unwrap();
        // let max = self
        //     .anchors
        //     .iter()
        //     .max_by(|a, b| {
        //         a.pos
        //             .x
        //             .partial_cmp(&b.pos.x)
        //             .unwrap()
        //             .then(a.pos.y.partial_cmp(&b.pos.y).unwrap())
        //     })
        //     .unwrap();
        (min, Vec2::new(max.x - min.x, max.y - min.y))
    }

    fn apply_actions(&mut self, response: &egui::Response, ui: &mut Ui) {
        println!("apply_actions: {:?}", self.dragging);

        // 获取鼠标位置（屏幕坐标）
        let screen_pos = match ui.ctx().input(|i| i.pointer.interact_pos()) {
            Some(p) => p,
            None => return,
        };
        // 转换为世界坐标
        let world_pos = self.canvas_state.to_canvas(screen_pos);
        // println!("world_pos: {:?}", world_pos);
        if response.drag_started() {
            println!("drag begin");
            if let Some((drag_type, _)) = self.hit_test(world_pos) {
                println!("drag begin: {:?}", drag_type);
                self.dragging = Some(drag_type);
            }
        }

        // 处理拖动开始
        // if response.clicked() {
        //     if let Some((drag_type, _)) = self.hit_test(world_pos) {
        //         println!("drag begin: {:?}", drag_type);
        //         self.dragging = Some(drag_type);
        //     }
        // }

        // 处理持续拖动
        if response.dragged() {
            println!("dragging: {:?}", self.dragging);
            if let Some(drag_type) = &self.dragging {
                println!("drag_type: {:?}", drag_type);
                match drag_type {
                    DragType::Anchor(index) => self.drag_anchor(*index, ui),
                    DragType::HandleIn(index) => self.drag_handle_in(*index, ui),
                    DragType::HandleOut(index) => self.drag_handle_out(*index, ui),
                }
            }
        }

        if response.drag_stopped() {
            println!("dragging released");
            self.dragging = None;
        }

        // 处理拖动结束
        // if response.clicked() {
        //     println!("dragging released");
        //     self.dragging = None;
        // }
    }

    /// 检测鼠标位置命中的元素
    fn hit_test(&self, world_pos: Pos2) -> Option<(DragType, usize)> {
        let hit_radius: f32 = 10.0 * self.canvas_state.scale;

        // 优先检测控制柄
        for (i, anchor) in self.anchors.iter().enumerate() {
            if (world_pos - anchor.handle_in).length() < hit_radius {
                return Some((DragType::HandleIn(i), i));
            }
            if (world_pos - anchor.handle_out).length() < hit_radius {
                return Some((DragType::HandleOut(i), i));
            }
        }

        // 检测锚点
        for (i, anchor) in self.anchors.iter().enumerate() {
            let offset = world_pos - anchor.pos;
            if offset.length() < hit_radius {
                println!("hit anchor: {:?}", i);
                return Some((DragType::Anchor(i), i));
            }
        }

        None
    }

    fn drag_anchor(&mut self, index: usize, ui: &mut Ui) {
        // println!("drag_anchor: {:?}", index);
        let anchor = &mut self.anchors[index];
        let delta = ui.input(|i| i.pointer.delta()) / self.canvas_state.scale;
        println!("delta: {:?}", delta);
        anchor.pos += delta;
        anchor.handle_in += delta;
        anchor.handle_out += delta;
        println!("anchor: {:?}", anchor.pos);
        // 如果锚点是平滑状态，需要强制更新控制柄
        if anchor.is_smooth {
            anchor.enforce_smooth();
        }
    }

    fn drag_handle_in(&mut self, index: usize, ui: &mut Ui) {
        let anchor = &mut self.anchors[index];
        let delta = ui.input(|i| i.pointer.delta()) / self.canvas_state.scale;
        anchor.handle_in += delta;

        if anchor.is_smooth {
            let mirror_delta = anchor.pos - (anchor.handle_in - anchor.pos);
            anchor.handle_out = mirror_delta;
        }
    }

    fn drag_handle_out(&mut self, index: usize, ui: &mut Ui) {
        let anchor = &mut self.anchors[index];
        let delta = ui.input(|i| i.pointer.delta()) / self.canvas_state.scale;
        anchor.handle_out += delta;

        if anchor.is_smooth {
            let mirror_delta = anchor.pos - (anchor.handle_out - anchor.pos);
            anchor.handle_in = mirror_delta;
        }
    }

    fn draw_bezier(&self, painter: &egui::Painter) {
        // 绘制外接矩形
        let rect = Rect::from_min_size(self.anchors[0].pos, self.desired_size().1);

        let screen_rect = self.canvas_state.to_screen_rect(rect);
        painter.rect(
            screen_rect,
            0.0,
            egui::Color32::TRANSPARENT,
            Stroke::new(1.0, egui::Color32::ORANGE),
        );

        // 绘制所有锚点和控制柄
        for anchor in &self.anchors {
            let screen_pos = self.canvas_state.to_screen(anchor.pos);
            let screen_handle_in = self.canvas_state.to_screen(anchor.handle_in);
            let screen_handle_out = self.canvas_state.to_screen(anchor.handle_out);
            let radius = 10.0 * self.canvas_state.scale;
            // println!("screen_pos: {:?}", screen_pos);
            // 绘制锚点
            painter.circle(
                screen_pos,
                radius,
                egui::Color32::GOLD,
                (1.0, egui::Color32::GOLD),
            );

            // 绘制控制柄线
            painter.line_segment(
                [screen_pos, screen_handle_in],
                (3.0, egui::Color32::LIGHT_BLUE),
            );
            painter.line_segment(
                [screen_pos, screen_handle_out],
                (3.0, egui::Color32::LIGHT_RED),
            );

            // 绘制控制柄点
            painter.circle(
                screen_handle_in,
                radius,
                egui::Color32::BLUE,
                (3.0, egui::Color32::LIGHT_BLUE),
            );
            painter.circle(
                screen_handle_out,
                radius,
                egui::Color32::RED,
                (3.0, egui::Color32::LIGHT_RED),
            );
        }

        // 绘制贝塞尔曲线路径
        if self.anchors.len() >= 2 {
            let mut path = Vec::new();
            for i in 0..self.anchors.len() - 1 {
                let screen_start = self.canvas_state.to_screen(self.anchors[i].pos);
                let screen_end = self.canvas_state.to_screen(self.anchors[i + 1].pos);
                let screen_cp1 = self.canvas_state.to_screen(self.anchors[i].handle_out);
                let screen_cp2 = self.canvas_state.to_screen(self.anchors[i + 1].handle_in);

                // 细分三次贝塞尔曲线为线段
                for t in 0..=100 {
                    let t = t as f32 / 100.0;
                    let point = cubic_bezier(screen_start, screen_cp1, screen_cp2, screen_end, t);
                    path.push(point);
                }
            }
            painter.add(Shape::line(path, Stroke::new(2.0, egui::Color32::GRAY)));
        }
    }
}

fn cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    t: f32,
) -> egui::Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    egui::pos2(
        u3 * p0.x + 3.0 * u2 * t * p1.x + 3.0 * u * t2 * p2.x + t3 * p3.x,
        u3 * p0.y + 3.0 * u2 * t * p1.y + 3.0 * u * t2 * p2.y + t3 * p3.y,
    )
}
