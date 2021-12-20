use connectorx::{
    prelude::*, sources::bigquery::BigQuerySource, sources::PartitionParser, sql::CXQuery,
};
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[test]
fn test_bigquery() {
    let _ = env_logger::builder().is_test(true).try_init();
    let dburl = env::var("BIGQUERY_URL").unwrap(); // TODO: hard-code

    let queries = [CXQuery::naked(
        "select * from `dataprep-bigquery.dataprep.test_table` where test_int < 2500 order by test_int",
    )];
    let rt = Arc::new(Runtime::new().unwrap());

    let mut source = BigQuerySource::new(rt, &dburl).unwrap();

    source.set_queries(&queries);
    source.fetch_metadata().unwrap();

    println!("{:?}", source.names());
    println!("{:?}", source.schema());

    let mut partitions = source.partition().unwrap();
    let mut partition = partitions.remove(0);
    partition.result_rows().expect("run query");
    assert_eq!(5, partition.nrows());
    assert_eq!(3, partition.ncols());

    let mut parser = partition.parser().unwrap();

    #[derive(Debug, PartialEq)]
    struct Row(Option<i64>, Option<String>, Option<f64>);
    let mut rows: Vec<Row> = Vec::new();

    loop {
        let (n, is_last) = parser.fetch_next().unwrap();
        for _i in 0..n {
            rows.push(Row(
                parser.produce().unwrap(),
                parser.produce().unwrap(),
                parser.produce().unwrap(),
            ));
        }
        if is_last {
            break;
        }
    }

    assert_eq!(
        vec![
            Row(Some(1), Some("str1".into()), Some(1.1)),
            Row(Some(2), Some("str2".into()), Some(2.2)),
            Row(Some(4), None, Some(-4.44)),
            Row(Some(5), Some("str05".into()), None),
            Row(Some(2333), None, None),
        ],
        rows
    );
}
