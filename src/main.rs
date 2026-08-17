use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use ppf_rust::{
    ImageType, PatchInfo, PpfCreatorOptions, PpfVersion, apply_patch, create_patch, inspect_patch,
    undo_patch,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply a PPF patch to a binary file
    Apply {
        /// The binary file to patch
        #[arg(short, long)]
        bin: String,
        /// The PPF patch file
        #[arg(short, long)]
        patch: String,
    },
    /// Undo a PPF3 patch from a binary file
    Undo {
        /// The binary file to unpatch
        #[arg(short, long)]
        bin: String,
        /// The PPF patch file with undo data
        #[arg(short, long)]
        patch: String,
    },
    /// Show patch information
    Info {
        /// The PPF patch file to inspect
        #[arg(short, long)]
        patch: String,
    },
    /// Create a new PPF patch from two binary files
    Create {
        /// The original unpatched binary file
        #[arg(short = 'O', long)]
        original: String,
        /// The modified binary file
        #[arg(short, long)]
        patched: String,
        /// The output PPF patch file
        #[arg(short, long)]
        output: String,
        /// Include undo data so the patch can be reversed
        #[arg(short = 'u', long)]
        undo: bool,
        /// Disable patch validation (blockcheck)
        #[arg(short = 'x', long)]
        disable_validation: bool,
        /// Image type: 0 = BIN, 1 = GI
        #[arg(short = 'i', long, default_value_t = 0)]
        imagetype: u8,
        /// Description text embedded in the patch header
        #[arg(short = 'd', long)]
        description: Option<String>,
        /// Path to a file_id.diz file to embed in the patch
        #[arg(short = 'f', long)]
        file_id: Option<String>,
    },
}

fn setup_record_progress(pb: &ProgressBar, total: usize) {
    pb.set_length(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} records ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
}

fn print_patch_info(info: &PatchInfo, action: &str, patch_path: &str, bin_path: &str) {
    let version_str = match info.version {
        PpfVersion::V1 => "1.0",
        PpfVersion::V2 => "2.0",
        PpfVersion::V3 => "3.0",
    };

    println!("Patchfile is a PPF{} patch.", version_str);
    println!("--- Patch Information ---");
    println!("Description : {}", info.description);

    if let Some(file_id) = &info.file_id {
        println!("File_id.diz : Available");
        println!("-------------------------");
        println!("{}", file_id.trim());
        println!("-------------------------");
    } else {
        println!("File_id.diz : Not available");
    }

    if info.version == PpfVersion::V3 {
        let image_type = match info.image_type {
            ImageType::Bin => "BIN (Standard)",
            ImageType::Gi => "GI (Global Image)",
        };
        println!("Image Type  : {}", image_type);
        println!(
            "Block Check : {}",
            if info.block_check {
                "Enabled"
            } else {
                "Disabled"
            }
        );
        println!(
            "Undo Data   : {}",
            if info.has_undo {
                "Available"
            } else {
                "Not available"
            }
        );
    }

    if !bin_path.is_empty() {
        println!("-------------------------");
        println!("{} patch '{}' to '{}'...", action, patch_path, bin_path);
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Apply { bin, patch } => {
            let pb = ProgressBar::new(0);
            let on_start_cb = |total: usize| {
                setup_record_progress(&pb, total);
            };
            let progress_cb = |records: usize| {
                pb.inc(records as u64);
            };

            let info = apply_patch(
                patch.as_str(),
                bin.as_str(),
                Some(&on_start_cb),
                Some(&progress_cb),
            )?;

            pb.finish_and_clear();
            print_patch_info(&info, "Applying", &patch, &bin);
            println!("Patch applied successfully.");
        }
        Commands::Undo { bin, patch } => {
            let pb = ProgressBar::new(0);
            let on_start_cb = |total: usize| {
                setup_record_progress(&pb, total);
            };
            let progress_cb = |records: usize| {
                pb.inc(records as u64);
            };

            let info = undo_patch(
                patch.as_str(),
                bin.as_str(),
                Some(&on_start_cb),
                Some(&progress_cb),
            )?;

            pb.finish_and_clear();
            print_patch_info(&info, "Undoing", &patch, &bin);
            println!("Patch undone successfully.");
        }
        Commands::Info { patch } => {
            let info = inspect_patch(patch.as_str())?;
            print_patch_info(&info, "Inspecting", &patch, "");
        }
        Commands::Create {
            original,
            patched,
            output,
            undo,
            disable_validation,
            imagetype,
            description,
            file_id,
        } => {
            let mut options = PpfCreatorOptions {
                undo_data: undo,
                block_check: !disable_validation,
                image_type: if imagetype == 1 {
                    ImageType::Gi
                } else {
                    ImageType::Bin
                },
                description: "PPF3 Patch".to_owned(),
                file_id: None,
            };

            if let Some(desc) = description {
                options.description = desc;
            }

            if let Some(file_id_path) = file_id {
                options.file_id = Some(std::fs::read(file_id_path)?);
            }

            let orig_len = std::fs::metadata(&original).map(|m| m.len()).unwrap_or(0);

            println!("Writing header... done.");
            println!("Finding differences... ");

            let pb = ProgressBar::new(orig_len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                    .progress_chars("#>-"),
            );

            let progress_cb = |bytes: usize| {
                pb.inc(bytes as u64);
            };

            let entries_found = create_patch(
                original.as_str(),
                patched.as_str(),
                output.as_str(),
                &options,
                Some(&progress_cb),
            )?;

            pb.finish_and_clear();

            println!("Progress: 100.00 % ({} entries found).", entries_found);
            println!("Done.");
        }
    }

    Ok(())
}
