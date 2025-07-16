use criterion::{black_box, criterion_group, criterion_main, Criterion};
use proof_messenger_relay::{Message, process_and_verify_message};
use proof_messenger_protocol::key::generate_keypair_with_seed;

// Helper function to create a valid test message
fn create_test_message(keypair_seed: u64, context: &[u8], body: &str) -> Message {
    let keypair = generate_keypair_with_seed(keypair_seed);
    let signature = keypair.sign(context);
    
    Message {
        sender: hex::encode(keypair.public.to_bytes()),
        context: hex::encode(context),
        body: body.to_string(),
        proof: hex::encode(signature.to_bytes()),
    }
}

fn proof_verification_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Create test messages with different sizes
    let small_context = b"small context";
    let small_message = create_test_message(42, small_context, "Small test message");
    
    let medium_context = b"medium sized context for verification benchmark";
    let medium_message = create_test_message(43, medium_context, "Medium sized test message for benchmarking the verification process");
    
    let large_context = b"large context with more data to sign and verify in the benchmark process to test performance with larger payloads";
    let large_body = "This is a much larger message body that contains significantly more text to process during the verification benchmark. This helps us understand how the system performs with larger payloads that might be more representative of real-world usage patterns where messages contain substantial content rather than just short test strings.";
    let large_message = create_test_message(44, large_context, large_body);
    
    // Benchmark small message verification
    c.bench_function("verify_small_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(process_and_verify_message(&small_message, None).await)
            })
        })
    });
    
    // Benchmark medium message verification
    c.bench_function("verify_medium_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(process_and_verify_message(&medium_message, None).await)
            })
        })
    });
    
    // Benchmark large message verification
    c.bench_function("verify_large_message", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(process_and_verify_message(&large_message, None).await)
            })
        })
    });
}

criterion_group!(benches, proof_verification_benchmark);
criterion_main!(benches);