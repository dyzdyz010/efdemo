use egui::Id;
use petgraph::graph::NodeIndex;

use crate::{
    canvas::CanvasState,
    graph::{
        anchor::{BezierAnchor, LineAnchor},
        render_info::NodeRenderInfo,
    },
    ui::{
        bezier::BezierEdge,
        line_edge::LineEdge,
        temp_edge::{TempEdge, TempEdgeTarget},
    },
};

use super::data::CanvasWidget;

pub fn draw_grid(ui: &mut egui::Ui, canvas_state: &CanvasState, screen_rect: egui::Rect) {
    // println!("draw_grid");
    let painter = ui.painter_at(screen_rect);

    // 基准网格间距（画布坐标系中的单位）
    let base_grid_size = 100.0;

    // 计算当前缩放下的网格像素大小
    let grid_pixels = base_grid_size * canvas_state.transform.scaling;

    // 计算网格级别
    let level_f = -(grid_pixels / base_grid_size).log2();
    // let level_f_offset = level_f + 0.5;
    let level = level_f.floor() as i32;
    // println!("level_f: {:?}", level_f);
    // println!("level: {:?}", level);
    // let level = level_f.floor() as i32;

    // 计算两个相邻级别的网格大小
    let grid_size_1 = base_grid_size * 2.0_f32.powi(level);
    let grid_size_2 = base_grid_size * 2.0_f32.powi(level + 1);

    // 计算两个级别的透明度
    let t = level_f.fract();
    let alpha_1;
    let alpha_2;
    if t >= 0.0 {
        alpha_1 = ((1.0 - t) * 255.0) as u8;
        alpha_2 = (t * 255.0) as u8;
    } else {
        let t = t.abs();
        alpha_1 = (t * 255.0) as u8;
        alpha_2 = ((1.0 - t) * 255.0) as u8;
    }

    // println!("alpha_1: {:?}", alpha_1);
    // println!("alpha_2: {:?}", alpha_2);

    // 定义网格颜色
    let grid_color_1 = egui::Color32::from_rgba_unmultiplied(100, 100, 100, alpha_1);
    let grid_color_2 = egui::Color32::from_rgba_unmultiplied(100, 100, 100, alpha_2);

    // 计算可见区域的边界（画布坐标）
    let min_canvas = canvas_state.to_canvas(screen_rect.min);
    let max_canvas = canvas_state.to_canvas(screen_rect.max);

    // 绘制第一级网格
    let x_start_1 = (min_canvas.x / grid_size_1).floor() as i32;
    let x_end_1 = (max_canvas.x / grid_size_1).ceil() as i32;
    let y_start_1 = (min_canvas.y / grid_size_1).floor() as i32;
    let y_end_1 = (max_canvas.y / grid_size_1).ceil() as i32;

    // let x_count_1 = x_end_1 - x_start_1;
    // let y_count_1 = y_end_1 - y_start_1;
    // println!("x_count_1: {:?}", x_count_1);
    // println!("y_count_1: {:?}", y_count_1);

    let mut path1 = Vec::new();
    let mut path2 = Vec::new();

    for x in x_start_1..=x_end_1 {
        let x_pos = x as f32 * grid_size_1;
        let p1 = canvas_state.to_screen(egui::Pos2::new(x_pos, min_canvas.y));
        let p2 = canvas_state.to_screen(egui::Pos2::new(x_pos, max_canvas.y));
        // painter.line_segment([p1, p2], (1.0, grid_color_1));
        path1.push([p1, p2]);
    }
    for y in y_start_1..=y_end_1 {
        let y_pos = y as f32 * grid_size_1;
        let p1 = canvas_state.to_screen(egui::Pos2::new(min_canvas.x, y_pos));
        let p2 = canvas_state.to_screen(egui::Pos2::new(max_canvas.x, y_pos));
        // painter.line_segment([p1, p2], (1.0, grid_color_1));
        path1.push([p1, p2]);
    }

    // 绘制第二级网格
    let x_start_2 = (min_canvas.x / grid_size_2).floor() as i32;
    let x_end_2 = (max_canvas.x / grid_size_2).ceil() as i32;
    let y_start_2 = (min_canvas.y / grid_size_2).floor() as i32;
    let y_end_2 = (max_canvas.y / grid_size_2).ceil() as i32;

    // let x_count_2 = x_end_2 - x_start_2;
    // let y_count_2 = y_end_2 - y_start_2;
    // println!("x_count_2: {:?}", x_count_2);
    // println!("y_count_2: {:?}", y_count_2);

    for x in x_start_2..=x_end_2 {
        let x_pos = x as f32 * grid_size_2;
        let p1 = canvas_state.to_screen(egui::Pos2::new(x_pos, min_canvas.y));
        let p2 = canvas_state.to_screen(egui::Pos2::new(x_pos, max_canvas.y));
        // painter.line_segment([p1, p2], (1.0, grid_color_2));
        path2.push([p1, p2]);
    }
    for y in y_start_2..=y_end_2 {
        let y_pos = y as f32 * grid_size_2;
        let p1 = canvas_state.to_screen(egui::Pos2::new(min_canvas.x, y_pos));
        let p2 = canvas_state.to_screen(egui::Pos2::new(max_canvas.x, y_pos));
        // painter.line_segment([p1, p2], (1.0, grid_color_2));
        path2.push([p1, p2]);
    }

    // 批量绘制第一级网格
    if !path1.is_empty() {
        let shape = egui::Shape::Vec(
            path1
                .into_iter()
                .map(|[p1, p2]| egui::Shape::line_segment([p1, p2], (1.0, grid_color_1)))
                .collect(),
        );
        painter.add(shape);
    }

    // 批量绘制第二级网格
    if !path2.is_empty() {
        let shape = egui::Shape::Vec(
            path2
                .into_iter()
                .map(|[p1, p2]| egui::Shape::line_segment([p1, p2], (1.0, grid_color_2)))
                .collect(),
        );
        painter.add(shape);
    }

    // 画坐标轴
    let axis_color = egui::Color32::RED;
    let origin = canvas_state.to_screen(egui::Pos2::ZERO);
    let x_axis_end = canvas_state.to_screen(egui::Pos2::new(1000.0, 0.0));
    let y_axis_end = canvas_state.to_screen(egui::Pos2::new(0.0, 1000.0));
    painter.line_segment([origin, x_axis_end], (2.0, axis_color));
    painter.line_segment([origin, y_axis_end], (2.0, axis_color));

    // 画一条线
    let line_start = canvas_state.to_screen(egui::Pos2::new(0.0, 0.0));
    let line_end = canvas_state.to_screen(egui::Pos2::new(1000.0, 1000.0));
    painter.line_segment([line_start, line_end], (2.0, egui::Color32::GREEN));

    // 画一个矩形
    let rect = egui::Rect::from_min_max(
        egui::Pos2::new(-500.0, -500.0),
        egui::Pos2::new(-150.0, -150.0),
    );
    let rect = canvas_state.to_screen_rect(rect);
    painter.rect(
        rect,
        egui::CornerRadius::same(5),
        egui::Color32::BLUE,
        egui::Stroke::new(2.0, egui::Color32::GREEN),
        egui::StrokeKind::Outside,
    );

    // 画一个圆
    // let circle_center = canvas_state.to_screen(egui::Pos2::new(500.0, 500.0));
    // // 将画布坐标系中的半径转换为屏幕坐标系中的半径
    // let circle_radius = 100.0 * canvas_state.scale;
    // painter.circle(
    //     circle_center,
    //     circle_radius,
    //     egui::Color32::RED,
    //     egui::Stroke::new(2.0, egui::Color32::GREEN),
    // );

    // let degree = 3;
    // let knots: Vec<f32> = (0..=control_points.len() + degree)
    //     .map(|i| i as f32)
    //     .collect();

    // let t_range = knots[degree]..=knots[control_points.len()];
    // let steps = 100;
    // let mut path = Vec::with_capacity(steps);

    // for step in 0..=steps {
    //     let t = t_range.start() + (t_range.end() - t_range.start()) * (step as f32 / steps as f32);
    //     if let Some(point) = de_boor_algorithm(&control_points, t, degree, &knots) {
    //         path.push(canvas_state.to_screen(point));
    //     }
    // }

    // painter.add(Shape::line(path, stroke));
}

impl CanvasWidget {
    pub fn hit_test_node(&self, ui: &mut egui::Ui, screen_pos: egui::Pos2) -> Option<NodeIndex> {
        self.graph_resource.read_graph(|graph| {
            graph.graph.node_indices().find(|&node_index| {
                let node_render_info: Option<NodeRenderInfo> = ui
                    .ctx()
                    .data(|d| d.get_temp(Id::new(node_index.index().to_string())));
                // println!("node_render_info: {:?}", node_render_info);
                if let Some(node_render_info) = node_render_info {
                    let node_screen_rect =
                        self.canvas_state_resource
                            .read_canvas_state(|canvas_state| {
                                canvas_state.to_screen_rect(node_render_info.canvas_rect)
                            });
                    if node_screen_rect.contains(screen_pos) {
                        return true;
                    }
                }
                false
            })
        })
    }

    pub fn make_temp_edge(&self, ui: &mut egui::Ui, node_index: NodeIndex) -> Option<TempEdge> {
        let node_render_info: Option<NodeRenderInfo> = ui
            .ctx()
            .data(|d| d.get_temp(Id::new(node_index.index().to_string())));
        // println!("node_render_info: {:?}", node_render_info);

        node_render_info.as_ref()?;

        let mouse_screen_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();
        let node_canvas_center = node_render_info.unwrap().canvas_center();

        let mouse_canvas_pos = self
            .canvas_state_resource
            .read_canvas_state(|canvas_state| canvas_state.to_canvas(mouse_screen_pos));

        Some(TempEdge {
            source: node_index,
            target: TempEdgeTarget::Point(mouse_canvas_pos),
            bezier_edge: BezierEdge::new(
                BezierAnchor::new_smooth(node_canvas_center),
                BezierAnchor::new_smooth(mouse_canvas_pos),
            )
            .with_control_anchors(vec![]),

            line_edge: LineEdge {
                source: LineAnchor::new(node_canvas_center),
                target: LineAnchor::new(mouse_canvas_pos),
            },
        })
    }
}
