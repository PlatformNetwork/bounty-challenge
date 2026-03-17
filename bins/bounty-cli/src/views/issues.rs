use crate::commands::issues::IssueList;
use crate::ui::components::{nav_bar, loading_spinner};
use crate::ui::primitives::br;
use crate::ui::styles::typography::{heading, text};
use crate::ui::styles::colors::text_muted;
use crate::ui::styles::spacing::{mt, mb};
use crate::state::user::UserState;
use crate::views::editor_settings::EditorSettingsPanel;
use crate::views::editor_settings::RenderWhitespace;

pub fn issues_view(user: &UserState, issues: &IssueList) -> String {
    let mut html = String::new();
    html.push_str(&nav_bar(user));
    html.push_str(&heading("Issues"));
    html.push_str(&br());
    // Render Whitespace settings dropdown for EditorSettingsPanel
    html.push_str(&format!(
        r#"<div class="mt-4">
            <label for="render-whitespace" class="block text-sm font-medium text-gray-700 mb-1">Render Whitespace</label>
            <select id="render-whitespace" name="render-whitespace" aria-label="Render Whitespace" class="block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md">
                <option value="none">None</option>
                <option value="boundary">Boundary</option>
                <option value="selection">Selection</option>
                <option value="all">All</option>
            </select>
        </div>"#
    ));
    // Existing issue list rendering
    html.push_str(&mt(2));
    html.push_str(&mb(2));
    if issues.items.is_empty() {
        html.push_str(&text_muted("No issues found."));
    } else {
        html.push_str("<ul class="divide-y divide-gray-200">\n");
        for issue in &issues.items {
            html.push_str(&format!(
                "<li class=\"py-4\"><a class=\"text-indigo-600 hover:text-indigo-900 font-medium\" href=\"{}\">{}</a><p class=\"text-gray-500 text-sm mt-1\">{}</p></li>\n",
                issue.url, issue.title, issue.body
            ));
        }
        html.push_str("</ul>\n");
    }
    html
}

