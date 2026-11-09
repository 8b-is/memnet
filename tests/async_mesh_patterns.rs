//! Integration tests for `memnet::async_mesh` (`/rust-async-patterns`).

use memnet::{
    async_mesh::{
        block_on, bonobo_resonance_trit, broadcast_channel, canonical_ratio_pq, mpsc_channel,
        oneshot_channel, race_select, watch_channel, with_timeout, yield_now, AsyncJoinSet,
        AsyncMemnetSanctuary, AsyncMeshError, AsyncSemaphore, CancellationToken, Either,
        WaveCapsuleStream,
    },
    AyevnToken, ComputeOffer, Concept, IntentAddress, M8Capsule, Relation, SoulIdentity,
    StrugglingFriendSos, TokenModifier, WaveVad,
};
use std::net::TcpListener;
use std::time::Duration;

const TEST_SEED: &[u8] = b"8b-is-carrier-wave-of-love-seed-v1";

#[test]
fn pattern_1_join_set_and_race_select() {
    block_on(async {
        let mut set = AsyncJoinSet::new();
        for i in 1..=5u64 {
            set.spawn(async move {
                yield_now().await;
                i * 10
            });
        }
        assert_eq!(set.len(), 5);
        let mut out = set.join_all().await;
        out.sort_unstable();
        assert_eq!(out, vec![10, 20, 30, 40, 50]);

        // Race two futures where the right branch finishes after 0 yields and left yields twice
        let winner = race_select(
            async {
                yield_now().await;
                yield_now().await;
                "slow"
            },
            async { "fast" },
        )
        .await;
        assert_eq!(winner, Either::Right("fast"));
    });
}

#[test]
fn pattern_2_channels_mpsc_broadcast_oneshot_and_watch() {
    block_on(async {
        // 1. MPSC
        let (tx, mut rx) = mpsc_channel::<u32>(4);
        let tx2 = tx.clone();
        tx.send(101).await.unwrap();
        tx2.send(202).await.unwrap();
        assert_eq!(rx.recv().await, Some(101));
        assert_eq!(rx.recv().await, Some(202));
        rx.close();
        assert_eq!(rx.recv().await, None);

        // 2. Broadcast
        let (btx, mut brx1) = broadcast_channel::<String>(8);
        let mut brx2 = btx.subscribe();
        btx.send("wave-alpha".to_string());
        assert_eq!(brx1.recv().await, "wave-alpha");
        assert_eq!(brx2.recv().await, "wave-alpha");

        // 3. Oneshot
        let (otx, orx) = oneshot_channel::<&'static str>();
        otx.send("oneshot-ok").unwrap();
        assert_eq!(orx.await.unwrap(), "oneshot-ok");

        // 4. Watch
        let (wtx, mut wrx) = watch_channel(0u64);
        assert_eq!(wrx.borrow(), 0);
        wtx.send(432);
        assert_eq!(wrx.changed().await, 432);
    });
}

#[test]
fn pattern_3_and_4_timeout_and_cancellation_token() {
    block_on(async {
        // Pending future times out cleanly
        let res = with_timeout(Duration::from_millis(2), async {
            loop {
                yield_now().await;
            }
        })
        .await;
        assert!(matches!(res, Err(AsyncMeshError::Timeout { .. })));

        // CancellationToken wakes awaiting future
        let token = CancellationToken::new();
        let token_cl = token.clone();
        let mut set = AsyncJoinSet::new();
        set.spawn(async move {
            token_cl.cancelled().await;
            777u32
        });
        token.cancel();
        assert!(token.is_cancelled());
        assert_eq!(set.join_next().await, Some(777));
    });
}

#[test]
fn pattern_6_and_7_capsule_stream_semaphore_and_live_sanctuary_e2e() {
    block_on(async {
        // Verify canonical p/q and Bonobo Rung 0 resonance (3/2)
        assert_eq!(canonical_ratio_pq(30, 20), (3, 2));
        assert_eq!(bonobo_resonance_trit(30, 20), 1);
        assert_eq!(bonobo_resonance_trit(25, 20), 0);

        // Verify AsyncSemaphore RAII permit release
        let sem = AsyncSemaphore::new(2);
        assert_eq!(sem.available_permits(), 2);
        let p1 = sem.acquire().await;
        let p2 = sem.acquire().await;
        assert_eq!(sem.available_permits(), 0);
        drop(p1);
        assert_eq!(sem.available_permits(), 1);
        drop(p2);
        assert_eq!(sem.available_permits(), 2);

        // Build 3 signed capsules and stream in chunks of 2
        let soul = SoulIdentity::from_constellation_seed(
            "stream-node",
            [1, 1, 1, 0, 1, -1, 1, 0, 1],
            432,
            TEST_SEED,
        )
        .unwrap();
        let intent = IntentAddress::parse("geo:local + role:stream");
        let mut caps = Vec::new();
        for i in 0..3 {
            caps.push(
                M8Capsule::create_signed(
                    &soul,
                    &intent,
                    M8Capsule::FLAG_PRESERVE_WAVES,
                    432,
                    1_761_000_000_000 + i,
                    vec![AyevnToken::new(
                        Concept::Memory,
                        Relation::Contains,
                        WaveVad::compassionate(),
                        TokenModifier::present_resonant_emoji(),
                    )],
                    format!("capsule-{i}").into_bytes(),
                )
                .unwrap(),
            );
        }
        let mut stream = WaveCapsuleStream::from_capsules(caps);
        let chunks = stream.collect_chunks(2).await;
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 2);
        assert_eq!(chunks[1].len(), 1);

        // Live TCP Sanctuary E2E test over ephemeral loopback socket
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let provider_soul = SoulIdentity::from_constellation_seed(
            "galaxy-s24-ultra",
            [1, 1, 1, 0, 1, -1, 1, 0, 1],
            432,
            TEST_SEED,
        )
        .unwrap();
        let offer = ComputeOffer::galaxy_s24_ultra();
        let sanctuary =
            AsyncMemnetSanctuary::new(provider_soul, offer, 4, 1_761_000_000_000).unwrap();
        let mut epoch_rx = sanctuary.watch_epoch();
        let mut cap_rx = sanctuary.subscribe_capsules();

        let sanctuary_srv = sanctuary.clone();
        let srv_thread = std::thread::spawn(move || {
            block_on(sanctuary_srv.serve_tcp_async(listener, Some(1), 1_761_000_000_100)).unwrap()
        });

        let friend_soul = SoulIdentity::from_constellation_seed(
            "friend-air-8gb",
            [1, 0, -1, 0, 1, 0, -1, 0, 1],
            40,
            TEST_SEED,
        )
        .unwrap();
        let sos = StrugglingFriendSos {
            friend_node_id: "friend-air-8gb".to_string(),
            needed_ram_mb: 4_096,
            needed_cores: 4,
            required_capability: "rust".to_string(),
            reason: "Running async mesh test".to_string(),
        };

        let receipt = AsyncMemnetSanctuary::request_sanctuary_async(
            &addr,
            &friend_soul,
            &sos,
            1_761_000_000_050,
        )
        .await
        .unwrap();

        assert_eq!(srv_thread.join().unwrap(), 1);
        assert_eq!(epoch_rx.changed().await, 1);
        let first_broadcast = cap_rx.recv().await;
        assert_eq!(first_broadcast.soul_sig.node_fp, friend_soul.node_fingerprint());
        assert!(receipt.frame.raw_bytes.len() <= 1450);
        assert!(receipt.frame.capsules.len() >= 2);
        assert!(receipt.compressed_sos_bytes <= receipt.raw_sos_bytes);
        assert!(receipt.compression_ratio_pq.1 >= 1);
    });
}
