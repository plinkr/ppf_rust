use ppf_rust::applier;
use ppf_rust::core::ImageType;
use ppf_rust::creator::{self, PpfCreatorOptions};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_apply_ppf1() {
    let mut bin = NamedTempFile::new().unwrap();
    let mut ppf = NamedTempFile::new().unwrap();

    let original_data = vec![0u8; 100];
    bin.write_all(original_data.as_slice()).unwrap();

    let mut ppf_data = Vec::new();
    ppf_data.extend_from_slice(b"PPF1");
    ppf_data.extend_from_slice(&[0, 0]);
    ppf_data.extend_from_slice(b"Test description                                  ");

    ppf_data.extend_from_slice(&10u32.to_le_bytes());
    ppf_data.push(3);
    ppf_data.extend_from_slice(&[1, 2, 3]);

    ppf.write_all(ppf_data.as_slice()).unwrap();

    let info = applier::apply_patch(ppf.path(), bin.path(), None, None).unwrap();
    assert_eq!(info.description, "Test description");
    assert_eq!(info.file_id, None);

    let result_data = std::fs::read(bin.path()).unwrap();
    assert_eq!(result_data[10], 1);
    assert_eq!(result_data[11], 2);
    assert_eq!(result_data[12], 3);
}

#[test]
fn test_apply_ppf2() {
    let mut bin = NamedTempFile::new().unwrap();
    let mut ppf = NamedTempFile::new().unwrap();

    let mut original_data = vec![0u8; 0x9320 + 2048];
    original_data[0x9320..0x9320 + 1024].fill(0xAA);
    bin.write_all(original_data.as_slice()).unwrap();

    let mut ppf_data = Vec::new();
    ppf_data.extend_from_slice(b"PPF2");
    ppf_data.extend_from_slice(&[0, 0]);
    ppf_data.extend_from_slice(b"PPF2 description                                  ");
    ppf_data.extend_from_slice(&(original_data.len() as u32).to_le_bytes());
    ppf_data.extend_from_slice(&original_data[0x9320..0x9320 + 1024]);

    ppf_data.extend_from_slice(&100u32.to_le_bytes());
    ppf_data.push(2);
    ppf_data.extend_from_slice(&[0x11, 0x22]);

    let file_id_text = "Release file_id for PPF2";
    ppf_data.extend_from_slice(b"@BEGIN_FILE_ID.DIZ");
    ppf_data.extend_from_slice(file_id_text.as_bytes());
    ppf_data.extend_from_slice(b"@END_FILE_ID.DIZ");
    ppf_data.extend_from_slice(&(file_id_text.len() as u32).to_le_bytes());

    ppf.write_all(ppf_data.as_slice()).unwrap();

    let info = applier::apply_patch(ppf.path(), bin.path(), None, None).unwrap();
    assert_eq!(info.description, "PPF2 description");
    assert_eq!(info.file_id.as_deref(), Some(file_id_text));

    let result_data = std::fs::read(bin.path()).unwrap();
    assert_eq!(result_data[100], 0x11);
    assert_eq!(result_data[101], 0x22);
}

#[test]
fn test_apply_and_undo_ppf3_manual() {
    let mut bin = NamedTempFile::new().unwrap();
    let mut ppf = NamedTempFile::new().unwrap();

    let original_data = vec![0u8; 100];
    bin.write_all(original_data.as_slice()).unwrap();

    let mut ppf_data = Vec::new();
    ppf_data.extend_from_slice(b"PPF30");
    ppf_data.push(2);
    ppf_data.extend_from_slice(b"Test description                                  ");
    ppf_data.push(0);
    ppf_data.push(0);
    ppf_data.push(1);
    ppf_data.push(0);

    ppf_data.extend_from_slice(&20u64.to_le_bytes());
    ppf_data.push(2);
    ppf_data.extend_from_slice(&[9, 9]);
    ppf_data.extend_from_slice(&[0, 0]);

    let diz = "My PPF3 File ID";
    ppf_data.extend_from_slice(b"@BEGIN_FILE_ID.DIZ");
    ppf_data.extend_from_slice(diz.as_bytes());
    ppf_data.extend_from_slice(b"@END_FILE_ID.DIZ");
    ppf_data.extend_from_slice(&(diz.len() as u16).to_le_bytes());

    ppf.write_all(ppf_data.as_slice()).unwrap();

    let info = applier::apply_patch(ppf.path(), bin.path(), None, None).unwrap();
    assert_eq!(info.description, "Test description");
    assert_eq!(info.file_id.as_deref(), Some(diz));

    let result_data = std::fs::read(bin.path()).unwrap();
    assert_eq!(result_data[20], 9);
    assert_eq!(result_data[21], 9);

    applier::undo_patch(ppf.path(), bin.path(), None, None).unwrap();

    let result_data = std::fs::read(bin.path()).unwrap();
    assert_eq!(result_data[20], 0);
    assert_eq!(result_data[21], 0);
}

#[test]
fn test_create_and_apply_ppf3_roundtrip() {
    let mut orig_file = NamedTempFile::new().unwrap();
    let mut mod_file = NamedTempFile::new().unwrap();
    let ppf_file = NamedTempFile::new().unwrap();

    let size = 64 * 1024;
    let mut orig_data = vec![0x33u8; size];
    let mut mod_data = vec![0x33u8; size];

    orig_data[50..55].copy_from_slice(&[1, 2, 3, 4, 5]);
    mod_data[50..55].copy_from_slice(&[10, 20, 30, 40, 50]);

    orig_data[40000..40300].fill(0x55);
    mod_data[40000..40300].fill(0x99);

    orig_file.write_all(&orig_data).unwrap();
    mod_file.write_all(&mod_data).unwrap();

    let options = PpfCreatorOptions {
        description: "Integration Roundtrip".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: true,
        file_id: Some(b"Roundtrip DIZ metadata".to_vec()),
    };

    let entries = creator::create_patch(
        orig_file.path(),
        mod_file.path(),
        ppf_file.path(),
        &options,
        None,
    )
    .unwrap();

    assert!(entries >= 2);

    let info = applier::apply_patch(ppf_file.path(), orig_file.path(), None, None).unwrap();
    assert_eq!(info.description, "Integration Roundtrip");
    assert_eq!(info.file_id.as_deref(), Some("Roundtrip DIZ metadata"));

    let applied_data = std::fs::read(orig_file.path()).unwrap();
    assert_eq!(applied_data, mod_data);

    applier::undo_patch(ppf_file.path(), orig_file.path(), None, None).unwrap();
    let undone_data = std::fs::read(orig_file.path()).unwrap();
    assert_eq!(
        undone_data[50..55],
        [1, 2, 3, 4, 5],
        "Undo should restore original data"
    );
    assert_eq!(
        undone_data[40000..40300],
        vec![0x55; 300],
        "Undo should restore original data"
    );
}

#[test]
fn test_create_patch_small_file_no_panic() {
    let mut orig_file = NamedTempFile::new().unwrap();
    let mut mod_file = NamedTempFile::new().unwrap();
    let ppf_file = NamedTempFile::new().unwrap();

    let orig_data = vec![0x11u8; 100];
    let mut mod_data = vec![0x11u8; 100];
    mod_data[10..15].copy_from_slice(&[1, 2, 3, 4, 5]);

    orig_file.write_all(&orig_data).unwrap();
    mod_file.write_all(&mod_data).unwrap();

    let options = PpfCreatorOptions {
        description: "Small file test".to_string(),
        image_type: ImageType::Bin,
        block_check: false,
        undo_data: true,
        file_id: None,
    };

    let count = creator::create_patch(
        orig_file.path(),
        mod_file.path(),
        ppf_file.path(),
        &options,
        None,
    )
    .unwrap();
    assert_eq!(count, 1);

    applier::apply_patch(ppf_file.path(), orig_file.path(), None, None).unwrap();
    let patched = std::fs::read(orig_file.path()).unwrap();
    assert_eq!(patched, mod_data);
}

#[test]
fn test_create_patch_small_file_with_blockcheck_does_not_panic() {
    let mut orig_file = NamedTempFile::new().unwrap();
    let mut mod_file = NamedTempFile::new().unwrap();
    let ppf_file = NamedTempFile::new().unwrap();

    let orig_data = vec![0x22u8; 50];
    let mut mod_data = vec![0x22u8; 50];
    mod_data[0] = 0x99;

    orig_file.write_all(&orig_data).unwrap();
    mod_file.write_all(&mod_data).unwrap();

    let options = PpfCreatorOptions {
        description: "Small with blockcheck".to_string(),
        image_type: ImageType::Bin,
        block_check: true,
        undo_data: false,
        file_id: None,
    };

    let count = creator::create_patch(
        orig_file.path(),
        mod_file.path(),
        ppf_file.path(),
        &options,
        None,
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_create_patch_stream_roundtrip() {
    let orig_data = vec![0x00u8; 1000];
    let mut mod_data = vec![0x00u8; 1000];
    mod_data[200..205].fill(0xFF);

    let orig_cursor = std::io::Cursor::new(orig_data.clone());
    let mod_cursor = std::io::Cursor::new(mod_data.clone());
    let mut patch_bytes = Vec::new();

    let options = PpfCreatorOptions {
        description: "Stream test".to_string(),
        image_type: ImageType::Bin,
        block_check: false,
        undo_data: true,
        file_id: None,
    };

    let entries =
        creator::create_patch_stream(orig_cursor, mod_cursor, &mut patch_bytes, &options, None)
            .unwrap();
    assert_eq!(entries, 1);

    let mut target_bin = NamedTempFile::new().unwrap();
    target_bin.write_all(&orig_data).unwrap();

    let mut ppf_file = NamedTempFile::new().unwrap();
    ppf_file.write_all(&patch_bytes).unwrap();

    applier::apply_patch(ppf_file.path(), target_bin.path(), None, None).unwrap();
    let result = std::fs::read(target_bin.path()).unwrap();
    assert_eq!(result, mod_data);
}
