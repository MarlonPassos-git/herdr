use super::*;

impl HeadlessServer {
    pub(super) fn handle_workspace_navigation_api(
        &mut self,
        id: String,
        target: &api::schema::PaneTarget,
    ) -> String {
        let Some((workspace, pane)) = self.app.parse_pane_id(&target.pane_id) else {
            return navigation_error(
                id,
                "pane_not_found",
                &format!(
                    "pane_id {:?} must identify an existing pane",
                    target.pane_id
                ),
            );
        };
        let Some(pane_id) = self.app.public_pane_id(workspace, pane) else {
            return navigation_error(
                id,
                "pane_not_found",
                &format!(
                    "pane_id {:?} must identify an existing pane",
                    target.pane_id
                ),
            );
        };
        let request = protocol::endpoint::EndpointWorkspaceNavigation {
            boot_id: self.client_shell_boot_id.clone(),
            pane_id,
        };
        let data = match serde_json::to_string(&request) {
            Ok(data) => data,
            Err(error) => return navigation_error(id, "serialization_failed", &error.to_string()),
        };
        // The server delivers an ephemeral intent; each client decides whether
        // the source pane is still selected. No presentation state is persisted.
        let clients: Vec<_> = self
            .clients
            .iter()
            .filter_map(|(&id, client)| {
                (matches!(client.mode, ClientConnectionMode::ClientShell)
                    && client.shell_surface_active
                    && client.writer.is_some())
                .then_some(id)
            })
            .collect();
        let mut delivered = false;
        for client in clients {
            delivered |= self.send_to_client(
                client,
                ServerMessage::EndpointControl {
                    kind: protocol::endpoint::WORKSPACE_NAVIGATION_KIND.into(),
                    data: data.clone(),
                },
            );
        }
        if !delivered {
            return navigation_error(
                id,
                "no_client_shell",
                "workspace navigation requires an attached client shell",
            );
        }
        serde_json::json!({"id": id, "result": {"type": "ok"}}).to_string()
    }
}

fn navigation_error(id: String, code: &str, message: &str) -> String {
    serde_json::json!({"id": id, "error": {"code": code, "message": message}}).to_string()
}
