use crate::application::Details;
use crate::components::admonition::{AdmonitionSeverity, admonition};
use crate::components::button::button;
use crate::components::icon_text::icon_text;
use crate::self_update::{SelfUpdate, VisibleSelfUpdateState};
use cntp_i18n::{tr, I18N_MANAGER};
use gpui::{
    App, BorrowAppContext, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px,
};

#[derive(IntoElement)]
pub struct UpdateNotification;

impl RenderOnce for UpdateNotification {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let self_update = cx.global::<SelfUpdate>();
        let details = cx.global::<Details>();
        let new_update = self_update.state();

        let application_name = I18N_MANAGER
            .locale()
            .messages
            .iter()
            .filter_map(|locale| {
                details
                    .generatable
                    .application_name
                    .resolve_language(locale)
            })
            .next()
            .unwrap_or_else(|| details.generatable.application_name.default_value());

        div()
            .p(px(4.))
            .child(match new_update.to_visible().unwrap() {
                VisibleSelfUpdateState::UpdateDownloaded { new_update } => admonition()
                    .severity(AdmonitionSeverity::Info)
                    .title(tr!("UPDATE_NOTIFICATION_TITLE", "Update Available"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(match &new_update.artifact_version {
                                None => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_TEXT_NO_VERSION",
                                        "A new version of {{application}} is available.",
                                        application = application_name
                                    )
                                }
                                Some(version) => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_TEXT",
                                        "{{application}} version {{version}} is available.",
                                        application = application_name,
                                        version = version
                                    )
                                }
                            })
                            .child(
                                button("perform-update")
                                    .child(icon_text(
                                        "package-upgrade",
                                        tr!(
                                            "UPDATE_NOTIFICATION_INSTALL_BUTTON",
                                            "Install Update and Restart"
                                        ),
                                    ))
                                    .on_click(|_, _, cx| {
                                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                                            self_update.install_downloaded_update(cx);
                                        })
                                    }),
                            ),
                    )
                    .into_any_element(),
                VisibleSelfUpdateState::UpdateInstalling { .. } => admonition()
                    .severity(AdmonitionSeverity::Info)
                    .title(tr!(
                        "UPDATE_NOTIFICATION_INSTALLING_TITLE",
                        "Installing Update"
                    ))
                    .child(div().flex().flex_col().gap(px(4.)).child(tr!(
                        "UPDATE_NOTIFICATION_INSTALLING",
                        "An update is being installed..."
                    )))
                    .into_any_element(),

                VisibleSelfUpdateState::RestartToUpdate { new_update } => admonition()
                    .severity(AdmonitionSeverity::Info)
                    .title(tr!("UPDATE_NOTIFICATION_TITLE"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(match &new_update.artifact_version {
                                None => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_RESTART_NO_VERSION",
                                        "Restart {{application}} to update to the latest version.",
                                        application = application_name
                                    )
                                }
                                Some(version) => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_RESTART",
                                        "Restart {{application}} to update to version {{version}}.",
                                        application = application_name,
                                        version = version
                                    )
                                }
                            })
                            .child(
                                button("restart-after-update")
                                    .child(icon_text(
                                        "system-reboot",
                                        tr!("UPDATE_NOTIFICATION_RESTART_BUTTON", "Restart"),
                                    ))
                                    .on_click(|_, _, cx| {
                                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                                            self_update.restart_application_after_update(cx);
                                        })
                                    }),
                            ),
                    )
                    .into_any_element(),
                VisibleSelfUpdateState::InstallStepFailed { new_update, error } => admonition()
                    .severity(AdmonitionSeverity::Error)
                    .title(tr!("UPDATE_NOTIFICATION_ERROR_TITLE", "Update Failed"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(match &new_update.artifact_version {
                                None => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_ERROR_NO_VERSION",
                                        "An update for {{application}} is available, \
                                        but it could not be installed due to an error.",
                                        application = application_name
                                    )
                                }
                                Some(version) => {
                                    tr!(
                                        "UPDATE_NOTIFICATION_ERROR",
                                        "Version {{version}} of  {{application}} is available, \
                                        but it could not be installed due to an error.",
                                        application = application_name,
                                        version = version
                                    )
                                }
                            })
                            .child(
                                button("retry-update")
                                    .child(icon_text(
                                        "view-refresh",
                                        tr!("UPDATE_NOTIFICATION_TRY_AGAIN_BUTTON", "Try Again"),
                                    ))
                                    .on_click(|_, _, cx| {
                                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                                            self_update.install_downloaded_update(cx);
                                        })
                                    }),
                            ),
                    )
                    .into_any_element(),
            })
    }
}
