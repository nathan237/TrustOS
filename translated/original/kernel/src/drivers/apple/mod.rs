//! Apple Hardware Drivers for TrustOS
//!
//! Apple Silicon (A-series / M-series) uses non-standard peripherals:
//! - **AIC** (Apple Interrupt Controller) instead of ARM GIC
//! - **Samsung-derived UART** instead of PL011
//! - **DART** (Device Address Resolution Table) instead of standard SMMU
//! - **PMGR** (Power Manager) for clock gating
//! - **SIO** for DMA
//!
//! These drivers are developed based on:
//! 1. Asahi Linux reverse engineering (public documentation)
//! 2. TrustOS iOS Recon tool output (tools/ios-recon/)
//! 3. Apple Device Tree dumps from jailbroken devices
//!
//! Reference: https://github.com/AsahiLinux/docs/wiki/HW-Overview

pub mod aic;
pub mod uart;
