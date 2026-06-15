use crate::theme::ThemeColors;
use crate::ui::atoms::input::text_input;
use egui::{Response, Ui};

fn filter_groups<'a>(query: &str, groups: &'a [String]) -> Vec<&'a String> {
    let q = query.to_lowercase();
    groups
        .iter()
        .filter(|g| q.is_empty() || g.to_lowercase().contains(&q))
        .collect()
}

pub fn group_input(ui: &mut Ui, value: &mut String, groups: &[String]) -> Response {
    let tc = ThemeColors::from_ui(ui);
    let popup_id = ui.make_persistent_id("group_input_popup");

    let resp = text_input(ui, value, "Local", None);

    if !resp.has_focus() {
        return resp;
    }

    let matching = filter_groups(value, groups);

    if matching.is_empty() {
        return resp;
    }

    let field_rect = resp.rect;

    egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(field_rect.min.x, field_rect.max.y + 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::new(1.0, tc.border))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::same(4))
                .show(ui, |ui| {
                    ui.set_min_width(field_rect.width().max(160.0));
                    ui.set_max_width(field_rect.width().max(240.0));
                    for group in &matching {
                        let selected = value.as_str() == group.as_str();
                        let row = ui.add(
                            egui::Button::selectable(
                                selected,
                                egui::RichText::new(*group).size(13.0).color(if selected {
                                    tc.button_primary_bg
                                } else {
                                    tc.text_primary
                                }),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .min_size(egui::vec2(0.0, 28.0)),
                        );
                        if row.clicked() {
                            *value = group.to_string();
                        }
                    }
                });
        });

    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn groups() -> Vec<String> {
        vec!["Local".into(), "Production".into(), "Staging".into()]
    }

    #[test]
    fn empty_query_returns_all_groups() {
        let g = groups();
        assert_eq!(filter_groups("", &g).len(), 3);
    }

    #[test]
    fn partial_query_filters_case_insensitively() {
        let g = groups();
        let local = "Local".to_string();
        assert_eq!(filter_groups("lo", &g), vec![&local]);
        assert_eq!(filter_groups("LO", &g), vec![&local]);
    }

    #[test]
    fn query_matching_multiple_groups_returns_all_matches() {
        let g = groups();
        let result = filter_groups("o", &g);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&&"Local".to_string()));
        assert!(result.contains(&&"Production".to_string()));
    }

    #[test]
    fn query_with_no_match_returns_empty() {
        let g = groups();
        assert!(filter_groups("zzz", &g).is_empty());
    }

    #[test]
    fn empty_groups_always_returns_empty() {
        assert!(filter_groups("", &[]).is_empty());
        assert!(filter_groups("lo", &[]).is_empty());
    }

    #[test]
    fn exact_match_is_included() {
        let g = groups();
        let local = "Local".to_string();
        assert_eq!(filter_groups("Local", &g), vec![&local]);
    }
}
