// SPDX-License-Identifier: MIT
// Copyright (c) 2026 WaveCord contributors

//! Raw parse throughput of the pure-Rust core.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wavecord_core::model::{LoadResult, ServerMessage};

fn one_track(i: usize) -> String {
    format!(
        r#"{{"encoded":"QAAAtwIADVVua25vd24gdGl0bGV{i}","info":{{"identifier":"id{i}","isSeekable":true,"author":"Author {i}","length":212000,"isStream":false,"position":0,"title":"Track number {i}","uri":"https://example.com/{i}.mp3","artworkUrl":null,"isrc":null,"sourceName":"http"}},"pluginInfo":{{}},"userData":{{}}}}"#
    )
}

fn search_result(n: usize) -> String {
    let tracks: Vec<String> = (0..n).map(one_track).collect();
    format!(r#"{{"loadType":"search","data":[{}]}}"#, tracks.join(","))
}

fn track_end_event() -> String {
    format!(
        r#"{{"op":"event","type":"TrackEndEvent","guildId":"123456789012345678","track":{},"reason":"finished"}}"#,
        one_track(1)
    )
}

fn bench(c: &mut Criterion) {
    let event = track_end_event();
    c.bench_function("parse_track_end_event", |b| {
        b.iter(|| {
            let msg: ServerMessage = serde_json::from_str(black_box(&event)).unwrap();
            black_box(msg);
        })
    });

    for n in [1usize, 50] {
        let payload = search_result(n);
        c.bench_function(&format!("parse_loadtracks_{n}_tracks"), |b| {
            b.iter(|| {
                let res: LoadResult = serde_json::from_str(black_box(&payload)).unwrap();
                black_box(res);
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
