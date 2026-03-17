/// Settings > Extensions rendering helpers
use yew::prelude::*;

/// Render the Indentation Guides setting row with proper accessibility association
pub fn render_indentation_guides_setting() -> Html {
    html! {
        <div class="setting-row">
            <label id="indentation-guides-label">
                { "Indentation Guides" }
            </label>
            <select
                class="setting-control"
                aria-labelledby="indentation-guides-label"
            >
                <option value="on">{ "On" }</option>
                <option value="off">{ "Off" }</option>
            </select>
        </div>
    }
}
