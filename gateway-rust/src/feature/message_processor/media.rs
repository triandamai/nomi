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
    pub tax: Option<f64>,
    pub service: Option<f64>,
    pub discount: Option<f64>,
    pub items: Vec<ExpenseItem>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseItem {
    pub name: String,
    pub quantity: i32,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceData {
    pub part_names: Vec<String>,
    pub service_details: String,
}
