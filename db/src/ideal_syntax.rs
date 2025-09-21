/// # Primitive types
/// - TEXT     (  String ) = Bytes checked to be utf8
/// - BYTES    ( Vec<u8> )
/// - BLOB<N>  ( [u8; N] )
/// # Typa aliases
/// - INT      ( integer ) = Blob<8>
/// - FLOAT    (     f64 ) = Blob<8>
/// - BOOL     (    bool ) = Blob<1>
///
/// Fields could be NULL if specified
///
///
/// Only one write connection could be established.
/// Immutable selection are unlimited.
///
/// Table could be server-owned(by default) and client-owned.
/// If server-owned, only client from you backend could modify it.
/// If client-owned, any connected cliend could modify data about themselves.
///
/// Row level security available. It allows clients to get mutable
/// connection to part of table with given PRIMARY KEY, so multiple
/// clients could write to 'same' table.
///
/// Transtaction(mutable connection) remains open as long as
/// client don't close it and stays alive. If it dies, transaction aborts.
///
/// Table could be APPENDONLY. Clients could have mupltiple connections to those tables and live queries are easier to implement.
///
/// When client want to make transaction:
/// 1. server locks table/tables(or part of it)
/// 2. client fully synchronizes to server
/// 3. client interact with his local tables
/// 4. client send all modifications to server
/// 5. server saves modifications
/// 6. server removes lock

#[derive(Column)]
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
    // or
    db.schema.define_table::<Mark>("marks");

    let mut tr = db.transaction(["marks"])?; // could be multiple tables
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
        marks.mut_key("id", strange_mark.id).update("value = 2")?; // can't modify selected property when using key method
        // eq to
        let mut ref_to_column = marks.mut_key("id", strange_mark.id);
        let val = ref_to_column.get()?;
        val.value = 2;
        ref_to_column.set(val); // logical alias to ref_to_column.update("id = _; subject_id = _;...")
    }

    tr.commit()?;

    println!("Hello, world!");
    Ok(())
}

struct MutTable<ColumnTy: Column>;
impl !Sync for MutTable<_> {}

impl<ColumnTy: Column> for MutTable<ColumnTy> {
    fn insert(&self, val: ColumnTy) -> Result<()>;
    fn select(&self, query: String, limit: Option<usize>) -> Result<Vec<ColumnTy>>;
    fn live_select(&self, query: String) -> Result<LiveSelection<ColumnTy>>;
    fn get(&self, field: String, val: IntoPrimitive) -> Result<ColumnWrapper<ColumnTy>>;
    fn live_get(&self, field: String, val: IntoPrimitive) -> Result<LiveRowSelection<ColumnTy>>;
    fn mut_key(&self, field: String, val: IntoPrimitive) -> Result<MutColumnWrapper<ColumnTy>>;
}
