use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaClassification {
    #[serde(rename = "EXPENSE_RECEIPT")]
    ExpenseReceipt,
    #[serde(rename = "MOTORCYCLE_MAINTENANCE")]
    MotorcycleMaintenance,
    #[serde(rename = "TECHNICAL_DOC")]
    TechnicalDoc,
    #[serde(rename = "NATURE")]
    Nature,
    #[serde(rename = "OTHER")]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseData {
    pub merchant: String,
    pub total: f64,
    pub items: Vec<String>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceData {
    pub part_names: Vec<String>,
    pub service_details: String,
}
