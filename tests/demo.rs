struct TestArchive {
    _temp_db_file: tempfile::NamedTempFile,
    arch: nbmarch::NBMStore,
}

fn create_test_archive() -> Result<TestArchive, Box<dyn std::error::Error>> {
    let temp_db_file = tempfile::NamedTempFile::new()?;
    let db_fname = temp_db_file.path();
    let arch = nbmarch::NBMStore::connect(db_fname)?;

    Ok(TestArchive {
        _temp_db_file: temp_db_file,
        arch,
    })
}

#[test]
fn test_connect() -> Result<(), Box<dyn std::error::Error>> {
    let temp_db_file = tempfile::NamedTempFile::new()?;
    let db_fname = temp_db_file.path();

    let _arch = nbmarch::NBMStore::connect(db_fname)?;

    Ok(())
}

#[test]
fn test_simple_validation() -> Result<(), Box<dyn std::error::Error>> {
    let arch = &create_test_archive()?.arch;

    let request_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(15, 15, 0);
    let valid_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(13, 0, 0);

    let validation = arch.validate_request("KMSO", request_time)?;
    assert_eq!(&validation.site.id, "KMSO");
    assert_eq!(&validation.site.name, "MISSOULA");
    assert_eq!(validation.initialization_time, valid_time);

    let validation = arch.validate_request("missoula", request_time)?;
    assert_eq!(&validation.site.id, "KMSO");
    assert_eq!(&validation.site.name, "MISSOULA");
    assert_eq!(validation.initialization_time, valid_time);

    let validation = arch.validate_request("logan", valid_time);
    assert!(validation.is_err());
    match validation {
        Err(nbmarch::Error::AmbiguousSite { .. }) => {}
        _ => panic!("Invalid error, should be ambiguous site"),
    }

    Ok(())
}

#[test]
fn test_retrieve() -> Result<(), Box<dyn std::error::Error>> {
    let arch = &create_test_archive()?.arch;

    let request_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(15, 15, 0);
    let valid_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(13, 0, 0);

    let validation = arch.validate_request("KMSO", request_time)?;
    assert_eq!(&validation.site.id, "KMSO");
    assert_eq!(&validation.site.name, "MISSOULA");
    assert_eq!(validation.initialization_time, valid_time);

    let _nbm = arch.retrieve(validation)?;

    Ok(())
}
