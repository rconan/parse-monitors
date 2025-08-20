# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the `psf` crate within the GMT (Giant Magellan Telescope) CFD analysis workspace. It generates Point Spread Function (PSF) visualizations using dome seeing turbulence data from CFD simulations combined with CRSEO optical modeling.

The parent workspace `parse-monitors` provides APIs for querying and processing GMT CFD Baseline databases with specialized tools for dome seeing analysis, pressure monitoring, and optical performance evaluation.

## Environment Setup

Required environment variables:
- `CFD_REPO`: Path to the CFD database root directory containing baseline cases
- `GMT_MODES_PATH`: Path to CEO mirror modes (for dome seeing analysis)

## Common Commands

### PSF Crate Specific
- `cargo run` - Generate short exposure PSF frames (default: 100 frames)  
- `cargo run -- --exposure short` - Generate individual PSF frames affected by turbulence
- `cargo run -- --exposure long` - Generate long exposure PSF (accumulated over 100 turbulence samples)
- `cargo build --release` - Release build with optimizations for faster PSF generation

### Workspace Commands
- `cargo test` - Run all tests (including monitors loading test)
- `cargo check` - Quick syntax and type checking
- `cargo run -p parse-monitors` - Run main library binary
- `cargo run --bin <binary_name>` - Run specific workspace binary

### Features
The workspace uses feature flags for different CFD years:
- `--features "2020"` - Use 2020 baseline data
- `--features "2021"` - Use 2021 baseline data  
- `--features "2025"` - Use 2025 baseline data (default)
- `--features "plot"` - Enable plotting capabilities (required for most binaries)

## Code Architecture

### PSF Crate Structure
- `src/main.rs` - PSF generation application with CRSEO integration
- Uses `CfdCase<CFD_YEAR>::colloquial(30, 0, "os", 7)` for standard 30° zenith, 0° azimuth, open sky, 7 m/s case
- Integrates `DomeSeeing` turbulence data with GMT optical model via CRSEO
- Generates frames with CUBEHELIX colormap normalization

### Key Types and Workflow
- `CfdCase<YEAR>` - Represents CFD simulation cases with zenith angle, azimuth, enclosure type, and wind speed
- `Baseline<YEAR>` - CFD database baseline for a specific year with iteration capabilities  
- `DomeSeeing` - Loads OPD turbulence data from CFD cases for ray tracing
- `Gmt` - CRSEO GMT telescope model for optical propagation
- `Source` - V-band source for PSF generation
- `Imaging`/`Detector` - Image detector with configurable pixel count and oversampling

### Output Files
- `psf.png` - Reference PSF without turbulence
- `frames/frame_XXXXXX.png` - Individual turbulence-affected PSF frames  
- `long_exposure_psf.png` - Accumulated PSF over multiple turbulence realizations

### Parent Workspace Structure
- `parse-monitors/` - Main library and CLI tools
- `domeseeing/` - Standalone dome seeing analysis
- `cfd_report/` - Report generation tools
- `windloads/` - Wind loading analysis
- `asm/` - Adaptive Secondary Mirror analysis
- `pressure-lambda/` - AWS Lambda pressure processing

## Data Processing Notes

PSF generation uses compressed CFD data files (monitors.csv.bz2) containing dome seeing turbulence volumes. The `DomeSeeing` iterator yields Optical Path Difference (OPD) maps that are applied to the GMT pupil for realistic atmospheric turbulence simulation.

Frame processing includes:
- Global min/max normalization across all frames for consistent visualization
- CUBEHELIX colormap application for scientific visualization
- Dashed white circle overlay showing atmospheric seeing diameter with 50% transparency
- Dotted white circle overlay showing GMT segment diffraction limit diameter with 50% transparency
- Progress bars for long-running computations
- Configurable detector size (default: 1000x1000 pixels with 4x oversampling)

Feature-gated compilation allows targeting specific CFD baseline years (2020, 2021, 2025) while maintaining a unified API.