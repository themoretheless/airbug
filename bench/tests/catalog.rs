//! Verification suite for workload catalog kernels (`#22–1000`), `#701` ABA-safe
//! Michael–Scott MPMC lock-free queue, and `#892–900` post-quantum/hash crypto primitives.
use airbug_bench::catalog::{self, CatalogCategory, crypto, lockfree};
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn all_catalog_kernels_have_unique_ids_and_execute() {
    let kernels = catalog::all_kernels();
    assert!(kernels.len() >= 24);
    let mut ids = BTreeSet::new();
    let mut slugs = BTreeSet::new();
    let mut cats = BTreeSet::new();
    for k in kernels {
        assert!(ids.insert(k.id), "duplicate catalog id {}", k.id);
        assert!(slugs.insert(k.slug), "duplicate catalog slug {}", k.slug);
        cats.insert(k.category);
        assert!(!k.title.is_empty());
        assert!(catalog::find_kernel(k.slug).is_some());
        assert!(catalog::find_kernel(&k.id.to_string()).is_some());
        let _ = std::hint::black_box((k.run)(16));
    }
    for cat in [
        CatalogCategory::Systems,
        CatalogCategory::RngHash,
        CatalogCategory::Algorithms,
        CatalogCategory::Concurrency,
        CatalogCategory::CryptoLattice,
        CatalogCategory::Observability,
    ] {
        assert!(
            !catalog::by_category(cat).is_empty(),
            "missing kernels for category {cat}"
        );
    }
}

#[test]
fn mpmc_queue_and_lockfree_primitives_conserve_items_under_contention() {
    let queue = Arc::new(lockfree::MpmcQueue::with_capacity(64));
    let producers = 4u64;
    let per_producer = 1000u64;
    let total = producers * per_producer;

    let popped = std::thread::scope(|scope| {
        let mut p_handles = Vec::new();
        for p in 0..producers {
            let q = queue.clone();
            p_handles.push(scope.spawn(move || {
                for i in 0..per_producer {
                    let item = p * per_producer + i;
                    while q.push(item).is_err() {
                        std::thread::yield_now();
                    }
                }
            }));
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let remaining = Arc::new(std::sync::atomic::AtomicU64::new(total));
        for _ in 0..4 {
            let q = queue.clone();
            let tx = tx.clone();
            let rem = remaining.clone();
            scope.spawn(move || {
                let mut local = Vec::new();
                while rem.load(std::sync::atomic::Ordering::Acquire) > 0 {
                    if let Some(v) = q.pop() {
                        local.push(v);
                        rem.fetch_sub(1, std::sync::atomic::Ordering::Release);
                    } else {
                        std::thread::yield_now();
                    }
                }
                tx.send(local).unwrap();
            });
        }
        drop(tx);

        for h in p_handles {
            h.join().unwrap();
        }
        let mut all = Vec::with_capacity(total as usize);
        for batch in rx {
            all.extend(batch);
        }
        all
    });

    let mut sorted = popped;
    sorted.sort_unstable();
    let expected: Vec<u64> = (0..total).collect();
    assert_eq!(sorted, expected, "MPMC queue lost or duplicated items");
    assert!(queue.pop().is_none());

    // Treiber stack contention conservation
    let stack = Arc::new(lockfree::TreiberStack::with_capacity(128));
    let mut stack_vals = std::thread::scope(|s| {
        for t in 0..4u64 {
            let st = stack.clone();
            s.spawn(move || {
                for i in 0..200u64 {
                    while st.push(t * 200 + i).is_err() {
                        std::thread::yield_now();
                    }
                }
            });
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let left = Arc::new(std::sync::atomic::AtomicU64::new(800));
        for _ in 0..4 {
            let st = stack.clone();
            let tx = tx.clone();
            let l = left.clone();
            s.spawn(move || {
                let mut local = Vec::new();
                while l.load(std::sync::atomic::Ordering::Acquire) > 0 {
                    if let Some(v) = st.pop() {
                        local.push(v);
                        l.fetch_sub(1, std::sync::atomic::Ordering::Release);
                    } else {
                        std::thread::yield_now();
                    }
                }
                tx.send(local).unwrap();
            });
        }
        drop(tx);
        rx.into_iter().flatten().collect::<Vec<u64>>()
    });
    stack_vals.sort_unstable();
    assert_eq!(stack_vals, (0..800).collect::<Vec<u64>>());

    // SPSC FIFO check
    let spsc = lockfree::SpscRing::with_power_of_two(8);
    for i in 0..8u64 {
        assert!(spsc.try_push(i));
    }
    assert!(!spsc.try_push(9));
    for i in 0..8u64 {
        assert_eq!(spsc.try_pop(), Some(i));
    }

    // SeqLock, TicketLock & StripedCounter
    let sl = lockfree::SeqLock::new((1u64, 2u64));
    sl.write((42, 84));
    assert_eq!(sl.read(), (42, 84));

    let tl = lockfree::TicketLock::new();
    assert_eq!(tl.with_lock(|| 7 + 5), 12);

    let sc = lockfree::StripedCounter::new();
    for i in 0..32 {
        sc.add(i, 3);
    }
    assert_eq!(sc.sum(), 96);
}

fn schoolbook_negacyclic(
    a: &[i32; crypto::KYBER_N],
    b: &[i32; crypto::KYBER_N],
) -> [i32; crypto::KYBER_N] {
    let q = crypto::KYBER_Q;
    let mut c = [0i32; crypto::KYBER_N];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            let prod = (ai * bj).rem_euclid(q);
            let idx = i + j;
            if idx < crypto::KYBER_N {
                c[idx] = (c[idx] + prod).rem_euclid(q);
            } else {
                c[idx - crypto::KYBER_N] = (c[idx - crypto::KYBER_N] - prod).rem_euclid(q);
            }
        }
    }
    c
}

#[test]
fn post_quantum_ntt_lwe_and_hash_signatures_roundtrip_and_reject_tampering() {
    // 1. Kyber NTT roundtrip + basemul equivalence to negacyclic convolution mod (X^256 + 1, 3329)
    let mut a = [0i32; crypto::KYBER_N];
    let mut b = [0i32; crypto::KYBER_N];
    for i in 0..crypto::KYBER_N {
        a[i] = (i as i32 * 37 + 11) % crypto::KYBER_Q;
        b[i] = (i as i32 * 91 + 5) % crypto::KYBER_Q;
    }
    let expected_conv = schoolbook_negacyclic(&a, &b);

    let mut a_hat = a;
    let mut b_hat = b;
    crypto::ntt_forward(&mut a_hat);
    crypto::ntt_forward(&mut b_hat);
    let mut inv_a = a_hat;
    crypto::ntt_inverse(&mut inv_a);
    assert_eq!(inv_a, a, "NTT inverse must recover input polynomial");

    let mut prod_hat = crypto::ntt_basemul(&a_hat, &b_hat);
    crypto::ntt_inverse(&mut prod_hat);
    assert_eq!(
        prod_hat, expected_conv,
        "NTT basemul must match negacyclic polynomial product"
    );

    // 2. Ring-LWE keygen / encrypt / decrypt roundtrip
    let (pk, sk) = crypto::lwe_keygen(&[0x42; 32]);
    for fill in [0x00u8, 0xff, 0xa5, 0x3c] {
        let msg = [fill; 32];
        let ct = crypto::lwe_encrypt(&pk, &msg, &[0x7f; 32]);
        let dec = crypto::lwe_decrypt(&sk, &ct);
        assert_eq!(dec, msg, "LWE decryption mismatch for fill={fill:#x}");
    }

    // 3. Lamport OTS + tamper rejection
    let (l_pk, l_sk) = crypto::lamport_keygen(&[0x19; 32]);
    let msg = b"airbug benchmark signature payload";
    let mut l_sig = crypto::lamport_sign(&l_sk, msg);
    assert!(crypto::lamport_verify(&l_pk, msg, &l_sig));
    assert!(!crypto::lamport_verify(&l_pk, b"tampered payload", &l_sig));
    l_sig[0][0] ^= 1;
    assert!(!crypto::lamport_verify(&l_pk, msg, &l_sig));

    // 4. Winternitz WOTS+ + tamper rejection
    let (w_pk, w_sk) = crypto::wots_keygen(&[0x01; 32], &[0x02; 32]);
    let mut w_sig = crypto::wots_sign(&w_sk, msg);
    assert!(crypto::wots_verify(&w_pk, msg, &w_sig));
    assert!(!crypto::wots_verify(
        &w_pk,
        b"tampered wots payload",
        &w_sig
    ));
    w_sig[10][3] ^= 1;
    assert!(!crypto::wots_verify(&w_pk, msg, &w_sig));

    // 5. Merkle proof + tamper rejection
    let leaves: Vec<[u8; 32]> = (0..16u8).map(|i| [i; 32]).collect();
    let (root, mut proof) = crypto::merkle_root_and_proof(&leaves, 9);
    assert!(crypto::merkle_verify(&root, &leaves[9], 9, &proof));
    assert!(!crypto::merkle_verify(&root, &leaves[8], 9, &proof));
    proof[0][0] ^= 1;
    assert!(!crypto::merkle_verify(&root, &leaves[9], 9, &proof));

    // 6. Constant-time helpers
    assert!(crypto::ct_eq(b"secret-tag-123", b"secret-tag-123"));
    assert!(!crypto::ct_eq(b"secret-tag-123", b"secret-tag-124"));
    assert!(!crypto::ct_eq(b"short", b"longer"));
    assert_eq!(crypto::ct_select_u64(true, 111, 222), 111);
    assert_eq!(crypto::ct_select_u64(false, 111, 222), 222);
}
