# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace for parsing and analyzing GMT (Giant Magellan Telescope) Computational Fluid Dynamics (CFD) data. The main library `parse-monitors` provides APIs for querying and processing GMT CFD Baseline databases with specialized tools for dome seeing analysis, pressure monitoring, and optical performance evaluation.

## Environment Setup

Required environment variables:
- `CFD_REPO`: Path to the CFD database root directory
- `GMT_MODES_PATH`: Path to CEO mirror modes (for dome seeing analysis)

## Common Commands

### Building and Running
- `cargo build` - Build all workspace members
- `cargo build --release` - Release build with optimizations
- `cargo run` - Run the default binary (parse-monitors)
- `cargo run --bin <binary_name>` - Run a specific binary
- `cargo check` - Quick syntax and type checking
- `cargo test` - Run all tests

### Specific Workspace Members
- `cargo run -p psf` - Run PSF (Point Spread Function) analysis
- `cargo run -p domeseeing` - Run dome seeing ray tracing
- `cargo run -p cfd_report` - Generate CFD reports
- `cargo run -p windloads` - Analyze wind loads

### Features
The library uses feature flags for different CFD years:
- `--features "2020"` - Use 2020 baseline data
- `--features "2021"` - Use 2021 baseline data  
- `--features "2025"` - Use 2025 baseline data (default)
- `--features "plot"` - Enable plotting capabilities

## Code Architecture

### Core Modules
- `cfd/` - CFD database models and case definitions (Baseline, CfdCase, year-specific implementations)
- `monitors/` - Force and moment monitoring data structures and loaders
- `pressure/` - Pressure analysis for mirrors and telescope components
- `domeseeing/` - Dome seeing turbulence analysis and ray tracing
- `temperature/` - Temperature field analysis
- `report/` - Report generation utilities

### Key Types
- `CfdCase<YEAR>` - Represents specific CFD simulation cases with zenith angle, azimuth, enclosure type, and wind speed
- `Baseline<YEAR>` - CFD database baseline for a specific year with iteration capabilities
- `Monitors`/`MonitorsLoader` - Load and process force/moment monitoring data
- `DomeSeeing` - Ray tracing through turbulence volumes for optical performance
- `Mirror` - Mirror-specific pressure and force analysis

### Workspace Structure
- `parse-monitors` - Main library and CLI tools
- `psf/` - Point spread function analysis using CRSEO
- `domeseeing/` - Standalone dome seeing analysis
- `cfd_report/` - Report generation tools
- `windloads/` - Wind loading analysis
- `asm/` - Adaptive Secondary Mirror analysis
- `gmacs/` - GMACS instrument specific analysis
- `htc-analysis/` - Heat transfer coefficient analysis
- `pressure-lambda/` - AWS Lambda pressure processing

## Data Processing Notes

The library handles compressed CFD data files (monitors.csv.bz2) and various data formats including pressure tables, temperature fields, and optical path difference maps. Most analysis requires setting the CFD_REPO environment variable to point to the database location.

Feature-gated compilation allows targeting specific CFD baseline years while maintaining a unified API. The workspace includes numerous specialized binaries for different analysis tasks, most requiring the "plot" feature for visualization capabilities.