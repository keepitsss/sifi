//! # Primitive types
//! - TEXT     (  String ) = Bytes checked to be utf8
//! - BYTES    ( Vec<u8> )
//! - BLOB<N>  ( [u8; N] )
//! # Typa aliases
//! - INT      ( integer ) = Blob<8>
//! - FLOAT    (     f64 ) = Blob<8>
//! - BOOL     (    bool ) = Blob<1>
//!
//! Fields could be NULL if specified

struct Mark {
    id: u64,
    subject_id: u64,
    value: u32,
    weight: u32,
    control_form_name: String,
    comment: Option<String>,
    point_date: Option<String>,
    date: String,
    is_point: bool,
    is_exam: bool,
}

fn main() -> anyhow::Result<()> {
    let db = Database::open()?;
    db.schema.modify(
        "
        CREATE TABLE marks (
            id                INT UNIQUE,
            subject_id        INT,
            value             INT,
            weight            INT,
            control_form_name TEXT,
            comment           TEXT COULD NULL,
            point_date        TEXT COULD NULL,
            date              TEXT,
            is_point          INT,
            is_exam           INT,
        )
        "
    )?;

    let mut tr = db.transaction()?;
    let mut marks = tr.table::<Mark>("marks").unwrap();
    marks.insert(Mark {
        id: todo!(),
        subject_id: todo!(),
        value: todo!(),
        weight: todo!(),
        control_form_name: todo!(),
        comment: todo!(),
        point_date: todo!(),
        date: todo!(),
        is_point: todo!(),
        is_exam: todo!(),
    })?;
    let strange_marks = marks.select().where("point_date IS SOME AND value != 2").list()?;
    for strange_mark in strange_marks {
        marks.select_mut().where(query!("id == {}", strange_mark.id)).update("value = 2")?;
        // or mb
        marks.key("id", strange_mark.id).update("value = 2")?; // can't modify selected property when using key method
        // eq to
        let ref_to_column = marks.key("id", strange_mark.id);
        let val = ref_to_column.get()?;
        val.value = 2;
        ref_to_column.set(val); // logical alias to ref_to_column.update("id = _; subject_id = _;...")
    }

    tr.commit()?;

    println!("Hello, world!");
    Ok(())
}
