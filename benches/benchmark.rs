//! yason benchmark

use bencher::{benchmark_group, benchmark_main, black_box, Bencher};
use std::cmp::Ordering;
use yason::{Array, ArrayRefBuilder, Number, Object, ObjectRefBuilder, PathExpression, YasonBuf};

fn bench_push_string(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, true).unwrap();
    bench.iter(|| {
        black_box(builder.push_string("string", "string").unwrap());
    })
}

fn bench_push_number(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, true).unwrap();
    let number = Number::from(123);
    bench.iter(|| {
        black_box(builder.push_number("string", number).unwrap());
    })
}

fn bench_push_bool(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, true).unwrap();
    bench.iter(|| {
        black_box(builder.push_bool("string", true).unwrap());
    })
}

fn bench_push_null(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, true).unwrap();
    bench.iter(|| {
        black_box(builder.push_null("string").unwrap());
    })
}

fn create_object(bytes: &mut Vec<u8>) -> Object {
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(bytes, 6, true).unwrap();
    // {key1: string, key2: 123, key3: true: key4: null, key5: [abc, false], key6: {key: true}}
    builder.push_string("key1", "string").unwrap();
    builder.push_number("key2", Number::from(123)).unwrap();
    builder.push_bool("key3", true).unwrap();
    builder.push_null("key4").unwrap();

    let mut array_builder = builder.push_array("key5", 2).unwrap();
    array_builder.push_string("abc").unwrap();
    array_builder.push_bool(false).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object("key6", 1, true).unwrap();
    object_builder.push_bool("key", true).unwrap();
    object_builder.finish().unwrap();

    let yason = builder.finish().unwrap();
    unsafe { Object::new_unchecked(yason) }
}

fn bench_object_read_string(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.string("key1").unwrap().unwrap());
    })
}

fn bench_object_read_number(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.number("key2").unwrap().unwrap());
    })
}

fn bench_object_read_bool(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.bool("key3").unwrap()).unwrap();
    })
}

fn bench_object_read_null(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.is_null("key4").unwrap().unwrap());
    })
}

fn bench_object_read_array(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.array("key5").unwrap().unwrap());
    })
}

fn bench_object_read_object(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let object = create_object(&mut bytes);
    bench.iter(|| {
        black_box(object.object("key6").unwrap().unwrap());
    })
}

fn create_array(bytes: &mut Vec<u8>) -> Array {
    bytes.clear();
    let mut builder = ArrayRefBuilder::try_new(bytes, 6).unwrap();
    // [string, 123, true, null, [abc, false], {key: true}]
    builder.push_string("string").unwrap();
    builder.push_number(Number::from(123)).unwrap();
    builder.push_bool(true).unwrap();
    builder.push_null().unwrap();

    let mut array_builder = builder.push_array(2).unwrap();
    array_builder.push_string("abc").unwrap();
    array_builder.push_bool(false).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object(1, true).unwrap();
    object_builder.push_bool("key", true).unwrap();
    object_builder.finish().unwrap();

    let yason = builder.finish().unwrap();
    unsafe { Array::new_unchecked(yason) }
}

fn bench_array_read_string(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.string(0).unwrap());
    })
}

fn bench_array_read_number(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.number(1).unwrap());
    })
}

fn bench_array_read_bool(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.bool(2).unwrap());
    })
}

fn bench_array_read_null(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.is_null(3).unwrap());
    })
}

fn bench_array_read_array(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.array(4).unwrap());
    })
}

fn bench_array_read_object(bench: &mut Bencher) {
    let mut bytes = Vec::with_capacity(1024);
    let array = create_array(&mut bytes);
    bench.iter(|| {
        black_box(array.object(5).unwrap());
    })
}

const KEYS_COUNT: usize = 5;

fn sort_init() -> (Vec<String>, Vec<u8>) {
    let keys = vec![
        "keykey".to_string(),
        "keyk".to_string(),
        "key".to_string(),
        "keyke".to_string(),
        "ke".to_string(),
    ];
    let bytes = Vec::with_capacity(1024);

    (keys, bytes)
}

fn sort_test(keys: &[String], bytes: &mut Vec<u8>, key_sorted: bool) {
    bytes.clear();
    let mut builder = ObjectRefBuilder::try_new(bytes, KEYS_COUNT as u16, key_sorted).unwrap();
    for key in keys.iter().take(KEYS_COUNT as usize) {
        builder.push_null(key.as_str()).unwrap();
    }
    builder.finish().unwrap();
}

fn bench_sort_no(bench: &mut Bencher) {
    let (mut keys, mut bytes) = sort_init();

    keys.sort_unstable_by(|a, b| match a.len().cmp(&b.len()) {
        Ordering::Equal => a.cmp(b),
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
    });

    bench.iter(|| sort_test(&keys, &mut bytes, true))
}

fn bench_sort_insert(bench: &mut Bencher) {
    let (keys, mut bytes) = sort_init();

    bench.iter(|| sort_test(&keys, &mut bytes, false))
}

fn bench_sort_new_builder(bench: &mut Bencher) {
    let (keys, mut bytes) = sort_init();

    fn inner(_keys: &[String], bytes: &mut Vec<u8>) {
        let _ = ObjectRefBuilder::try_new(bytes, KEYS_COUNT as u16, true).unwrap();
    }

    bench.iter(|| inner(&keys, &mut bytes))
}

fn bench_query(bench: &mut Bencher) {
    let input = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;
    let path = "$.key4[last - 20, last - 2, 2 to 4, 0].*[0]..key2.type()";
    let yason_buf = YasonBuf::parse(input).unwrap();
    let yason = yason_buf.as_ref();
    let path = str::parse::<PathExpression>(path).unwrap();

    bench.iter(|| path.query(yason, true, None, None).unwrap())
}

benchmark_group!(
    yason_benches,
    bench_push_string,
    bench_push_number,
    bench_push_bool,
    bench_push_null,
    bench_sort_no,
    bench_sort_insert,
    bench_sort_new_builder,
    bench_object_read_string,
    bench_object_read_number,
    bench_object_read_bool,
    bench_object_read_null,
    bench_object_read_array,
    bench_object_read_object,
    bench_array_read_string,
    bench_array_read_number,
    bench_array_read_bool,
    bench_array_read_null,
    bench_array_read_array,
    bench_array_read_object,
    bench_query,
);

benchmark_main!(yason_benches);
