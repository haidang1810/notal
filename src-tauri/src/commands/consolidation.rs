// Manual consolidation trigger command for the Tauri frontend

use tauri::State;

use crate::db::DbState;
use crate::memory::consolidation::{
    run_consolidation, ConsolidationConfig, ConsolidationReport,
};
use crate::memory::decay::DecayConfig;
use crate::memory::promotion::PromotionThresholds;
use crate::LlmState;

/// Tauri command: run a full consolidation cycle immediately on demand.
/// Returns a `ConsolidationReport` serialised as JSON for the frontend.
#[tauri::command]
pub async fn trigger_consolidation(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
) -> Result<ConsolidationReport, String> {
    run_consolidation(
        db.inner(),
        llm.inner(),
        &DecayConfig::default(),
        &PromotionThresholds::default(),
        &ConsolidationConfig::default(),
    )
    .await
}
