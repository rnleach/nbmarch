#[test]
fn test_download() -> Result<(), Box<dyn std::error::Error>> {
    let temp_db_file = tempfile::NamedTempFile::new()?;
    dbg!(&temp_db_file);
    let db_fname = temp_db_file.path();
    dbg!(&db_fname);

    let arch = nbmarch::NBMStore::connect(db_fname)?;
    dbg!();

    let valid_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(13, 0, 0);

    let validation = arch.validate_request("KMSO", valid_time)?;
    println!("{:?}", validation);

    let validation = arch.validate_request("missoula", valid_time)?;
    println!("{:?}", validation);

    let validation = arch.validate_request("logan", valid_time);
    assert!(validation.is_err());
    println!("{:?}", validation);

    Ok(())
}
