#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Struct to represent an invoice token
#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    pub invoice_id: u64,
    pub seller: Address,           // Business selling the invoice
    pub buyer: Address,            // Optional buyer who purchased the invoice
    pub invoice_amount: i64,       // Original invoice amount
    pub discount_rate: u64,        // Discount percentage (e.g., 5 means 5%)
    pub selling_price: i64,        // Discounted price for selling
    pub is_sold: bool,             // Status of the invoice
    pub creation_time: u64,        // Timestamp when invoice was listed
}

// Counter for generating unique invoice IDs
const INVOICE_COUNT: Symbol = symbol_short!("INV_CNT");

// Mapping invoice ID to Invoice struct
#[contracttype]
pub enum InvoiceBook {
    Invoice(u64)
}

// Platform statistics
#[contracttype]
#[derive(Clone)]
pub struct PlatformStats {
    pub total_invoices: u64,
    pub active_invoices: u64,
    pub sold_invoices: u64,
    pub total_volume: i64,
}

const STATS: Symbol = symbol_short!("STATS");

#[contract]
pub struct InvoiceFactoringContract;

#[contractimpl]
impl InvoiceFactoringContract {
    
    // Function to list an unpaid invoice for sale
    pub fn list_invoice(
        env: Env, 
        seller: Address, 
        invoice_amount: i64, 
        discount_rate: u64
    ) -> u64 {
        // Authenticate the seller
        seller.require_auth();
        
        // Generate unique invoice ID
        let mut invoice_count: u64 = env.storage().instance().get(&INVOICE_COUNT).unwrap_or(0);
        invoice_count += 1;
        
        // Calculate selling price with discount
        let discount_amount = (invoice_amount * discount_rate as i64) / 100;
        let selling_price = invoice_amount - discount_amount;
        
        // Get current timestamp
        let time = env.ledger().timestamp();
        
        // Create new invoice
        let new_invoice = Invoice {
            invoice_id: invoice_count,
            seller: seller.clone(),
            buyer: Address::from_string(&String::from_str(&env, "unassigned")),
            invoice_amount,
            discount_rate,
            selling_price,
            is_sold: false,
            creation_time: time,
        };
        
        // Update platform statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.total_invoices += 1;
        stats.active_invoices += 1;
        
        // Store the invoice
        env.storage().instance().set(&InvoiceBook::Invoice(invoice_count), &new_invoice);
        env.storage().instance().set(&INVOICE_COUNT, &invoice_count);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Invoice listed with ID: {}", invoice_count);
        invoice_count
    }
    
    // Function to purchase an invoice token
    pub fn purchase_invoice(env: Env, invoice_id: u64, buyer: Address) {
        // Authenticate the buyer
        buyer.require_auth();
        
        // Retrieve the invoice
        let mut invoice = Self::get_invoice(env.clone(), invoice_id);
        
        // Validate that invoice exists and is not sold
        if invoice.invoice_id == 0 {
            log!(&env, "Invoice not found");
            panic!("Invoice not found");
        }
        
        if invoice.is_sold {
            log!(&env, "Invoice already sold");
            panic!("Invoice already sold");
        }
        
        // Update invoice with buyer information
        invoice.buyer = buyer.clone();
        invoice.is_sold = true;
        
        // Update platform statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.active_invoices -= 1;
        stats.sold_invoices += 1;
        stats.total_volume += invoice.selling_price;
        
        // Store updated invoice and stats
        env.storage().instance().set(&InvoiceBook::Invoice(invoice_id), &invoice);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Invoice {} purchased by buyer", invoice_id);
    }
    
    // Function to view a specific invoice by ID
    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        let key = InvoiceBook::Invoice(invoice_id);
        
        env.storage().instance().get(&key).unwrap_or(Invoice {
            invoice_id: 0,
            seller: Address::from_string(&String::from_str(&env, "none")),
            buyer: Address::from_string(&String::from_str(&env, "none")),
            invoice_amount: 0,
            discount_rate: 0,
            selling_price: 0,
            is_sold: false,
            creation_time: 0,
        })
    }
    
    // Function to get platform statistics
    pub fn get_platform_stats(env: Env) -> PlatformStats {
        env.storage().instance().get(&STATS).unwrap_or(PlatformStats {
            total_invoices: 0,
            active_invoices: 0,
            sold_invoices: 0,
            total_volume: 0,
        })
    }
}