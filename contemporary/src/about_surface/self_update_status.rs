use crate::components::button::button;
use crate::components::icon_text::icon_text;
use crate::components::layer::layer;
use crate::components::spinner::spinner;
use crate::self_update::{SelfUpdate, SelfUpdateState};
use cntp_i18n::tr;
use gpui::{
    App, BorrowAppContext, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px,
};

#[derive(IntoElement)]
pub struct SelfUpdateStatus;

impl RenderOnce for SelfUpdateStatus {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let self_update = cx.global::<SelfUpdate>();

        let (text, button) = match self_update.state() {
            SelfUpdateState::Idle => (
                tr!("ABOUT_UPDATE_NONE", "You are up to date."),
                button("check-for-updates-again")
                    .child(icon_text(
                        "view-refresh",
                        tr!("ABOUT_UPDATE_CHECK_AGAIN", "Check for updates"),
                    ))
                    .on_click(|_, _, cx| {
                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                            self_update.check_for_updates(true, cx)
                        })
                    })
                    .into_any_element(),
            ),
            SelfUpdateState::UpdateChecking => (
                tr!("ABOUT_UPDATE_CHECKING", "Checking for updates..."),
                spinner().into_any_element(),
            ),
            SelfUpdateState::UpdateAvailableBackground { .. } => (
                tr!("ABOUT_UPDATE_AVAILABLE", "An update is available."),
                button("download")
                    .child(icon_text(
                        "cloud-download",
                        tr!("ABOUT_UPDATE_DOWNLOAD", "Download Update"),
                    ))
                    .on_click(|_, _, cx| {
                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                            self_update.start_update_download(cx)
                        })
                    })
                    .into_any_element(),
            ),
            SelfUpdateState::UpdateDownloadingInBackground { new_update } => (
                match new_update.artifact_version.as_ref() {
                    None => {
                        tr!(
                            "ABOUT_UPDATE_DOWNLOADING",
                            "An update is being downloaded..."
                        )
                    }
                    Some(version) => {
                        tr!(
                            "ABOUT_UPDATE_DOWNLOADING_VERSION",
                            "Version {{version}} is being downloaded...",
                            version = version
                        )
                    }
                },
                spinner().into_any_element(),
            ),
            SelfUpdateState::UpdateDownloaded { new_update }
            | SelfUpdateState::UpdateDownloadedInBackground { new_update } => (
                tr!("ABOUT_UPDATE_READY", "An update is ready to install."),
                button("download")
                    .child(icon_text(
                        "package-upgrade",
                        tr!("UPDATE_NOTIFICATION_INSTALL_BUTTON"),
                    ))
                    .on_click(|_, _, cx| {
                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                            self_update.install_downloaded_update(cx)
                        })
                    })
                    .into_any_element(),
            ),
            SelfUpdateState::UpdateInstalling { new_update } => (
                tr!("ABOUT_UPDATE_INSTALLING", "An update is being installed."),
                spinner().into_any_element(),
            ),
            SelfUpdateState::RestartToUpdate { new_update } => (
                tr!("ABOUT_UPDATE_INSTALLED", "An update has been installed."),
                button("download")
                    .child(icon_text(
                        "system-reboot",
                        tr!("UPDATE_NOTIFICATION_RESTART_BUTTON"),
                    ))
                    .on_click(|_, _, cx| {
                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                            self_update.restart_application_after_update(cx)
                        })
                    })
                    .into_any_element(),
            ),
            SelfUpdateState::InstallStepFailed { new_update, .. } => (
                tr!(
                    "ABOUT_UPDATE_FAILED",
                    "An update was unable to be installed."
                ),
                button("retry-update")
                    .child(icon_text(
                        "view-refresh",
                        tr!("UPDATE_NOTIFICATION_TRY_AGAIN_BUTTON"),
                    ))
                    .on_click(|_, _, cx| {
                        cx.update_global::<SelfUpdate, _>(|self_update, cx| {
                            self_update.install_downloaded_update(cx)
                        })
                    })
                    .into_any_element(),
            ),
        };

        layer().child(
            div()
                .p(px(4.))
                .flex()
                .items_center()
                .child(text)
                .child(div().flex_grow(1.))
                .child(button),
        )
    }
}
