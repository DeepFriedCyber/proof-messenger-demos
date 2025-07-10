use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use proof_messenger_protocol::prelude::*;

fn benchmark_keypair_generation(c: &mut Criterion) {
    c.bench_function("keypair_generation", |b| {
        b.iter(|| {
            let keypair = KeyPair::generate().expect("Failed to generate keypair");
            black_box(keypair);
        })
    });
}

fn benchmark_message_signing(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    let message_sizes = [100, 1000, 10000, 100000];
    
    let mut group = c.benchmark_group("message_signing");
    
    for size in message_sizes.iter() {
        let message_data = vec![0u8; *size];
        
        group.bench_with_input(BenchmarkId::new("sign", size), size, |b, _| {
            b.iter(|| {
                let signature = keypair.sign(&message_data).expect("Failed to sign message");
                black_box(signature);
            })
        });
    }
    
    group.finish();
}

fn benchmark_signature_verification(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    let message_sizes = [100, 1000, 10000, 100000];
    
    let mut group = c.benchmark_group("signature_verification");
    
    for size in message_sizes.iter() {
        let message_data = vec![0u8; *size];
        let signature = keypair.sign(&message_data).expect("Failed to sign message");
        
        group.bench_with_input(BenchmarkId::new("verify", size), size, |b, _| {
            b.iter(|| {
                let is_valid = keypair.public_key().verify(&message_data, &signature)
                    .expect("Failed to verify signature");
                black_box(is_valid);
            })
        });
    }
    
    group.finish();
}

fn benchmark_proof_creation(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    let data_sizes = [32, 256, 1024, 4096];
    
    let mut group = c.benchmark_group("proof_creation");
    
    for size in data_sizes.iter() {
        let proof_data = vec![0u8; *size];
        
        group.bench_with_input(BenchmarkId::new("unsigned", size), size, |b, _| {
            b.iter(|| {
                let proof = Proof::new(ProofType::Message, proof_data.clone())
                    .expect("Failed to create proof");
                black_box(proof);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("signed", size), size, |b, _| {
            b.iter(|| {
                let proof = Proof::new_signed(ProofType::Message, proof_data.clone(), &keypair)
                    .expect("Failed to create signed proof");
                black_box(proof);
            })
        });
    }
    
    group.finish();
}

fn benchmark_proof_verification(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    let data_sizes = [32, 256, 1024, 4096];
    
    let mut group = c.benchmark_group("proof_verification");
    
    for size in data_sizes.iter() {
        let proof_data = vec![0u8; *size];
        let unsigned_proof = Proof::new(ProofType::Message, proof_data.clone())
            .expect("Failed to create unsigned proof");
        let signed_proof = Proof::new_signed(ProofType::Message, proof_data, &keypair)
            .expect("Failed to create signed proof");
        
        group.bench_with_input(BenchmarkId::new("unsigned", size), size, |b, _| {
            b.iter(|| {
                let is_valid = ProofVerifier::verify(&unsigned_proof)
                    .expect("Failed to verify unsigned proof");
                black_box(is_valid);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("signed", size), size, |b, _| {
            b.iter(|| {
                let is_valid = ProofVerifier::verify(&signed_proof)
                    .expect("Failed to verify signed proof");
                black_box(is_valid);
            })
        });
    }
    
    group.finish();
}

fn benchmark_message_creation(c: &mut Criterion) {
    let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
    let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
    let content_sizes = [10, 100, 1000, 10000];
    
    let mut group = c.benchmark_group("message_creation");
    
    for size in content_sizes.iter() {
        let content = "a".repeat(*size);
        
        group.bench_with_input(BenchmarkId::new("unsigned", size), size, |b, _| {
            b.iter(|| {
                let message = MessageBuilder::new()
                    .sender(sender_keypair.public_key().clone())
                    .recipient(recipient_keypair.public_key().clone())
                    .content(content.clone())
                    .build()
                    .expect("Failed to build message");
                black_box(message);
            })
        });
        
        group.bench_with_input(BenchmarkId::new("signed", size), size, |b, _| {
            b.iter(|| {
                let message = MessageBuilder::new()
                    .sender(sender_keypair.public_key().clone())
                    .recipient(recipient_keypair.public_key().clone())
                    .content(content.clone())
                    .sign_with(&sender_keypair)
                    .expect("Failed to set signing keypair")
                    .build()
                    .expect("Failed to build signed message");
                black_box(message);
            })
        });
    }
    
    group.finish();
}

fn benchmark_message_verification(c: &mut Criterion) {
    let sender_keypair = KeyPair::generate().expect("Failed to generate sender keypair");
    let recipient_keypair = KeyPair::generate().expect("Failed to generate recipient keypair");
    
    let signed_message = MessageBuilder::new()
        .sender(sender_keypair.public_key().clone())
        .recipient(recipient_keypair.public_key().clone())
        .content("Test message for verification benchmark".to_string())
        .sign_with(&sender_keypair)
        .expect("Failed to set signing keypair")
        .build()
        .expect("Failed to build signed message");
    
    c.bench_function("message_verification", |b| {
        b.iter(|| {
            let is_valid = signed_message.verify_signature()
                .expect("Failed to verify message signature");
            black_box(is_valid);
        })
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    
    let message = MessageBuilder::new()
        .sender(keypair.public_key().clone())
        .recipient(keypair.public_key().clone())
        .content("Serialization benchmark message".to_string())
        .sign_with(&keypair)
        .expect("Failed to set signing keypair")
        .build()
        .expect("Failed to build message");
    
    let proof = Proof::new_signed(
        ProofType::Message,
        b"Serialization benchmark proof data".to_vec(),
        &keypair,
    ).expect("Failed to create proof");
    
    let mut group = c.benchmark_group("serialization");
    
    group.bench_function("message_json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&message)
                .expect("Failed to serialize message to JSON");
            black_box(json);
        })
    });
    
    group.bench_function("message_json_deserialize", |b| {
        let json = serde_json::to_string(&message)
            .expect("Failed to serialize message to JSON");
        b.iter(|| {
            let deserialized: Message = serde_json::from_str(&json)
                .expect("Failed to deserialize message from JSON");
            black_box(deserialized);
        })
    });
    
    group.bench_function("message_binary_serialize", |b| {
        b.iter(|| {
            let binary = bincode::serialize(&message)
                .expect("Failed to serialize message to binary");
            black_box(binary);
        })
    });
    
    group.bench_function("message_binary_deserialize", |b| {
        let binary = bincode::serialize(&message)
            .expect("Failed to serialize message to binary");
        b.iter(|| {
            let deserialized: Message = bincode::deserialize(&binary)
                .expect("Failed to deserialize message from binary");
            black_box(deserialized);
        })
    });
    
    group.bench_function("proof_json_serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&proof)
                .expect("Failed to serialize proof to JSON");
            black_box(json);
        })
    });
    
    group.bench_function("proof_json_deserialize", |b| {
        let json = serde_json::to_string(&proof)
            .expect("Failed to serialize proof to JSON");
        b.iter(|| {
            let deserialized: Proof = serde_json::from_str(&json)
                .expect("Failed to deserialize proof from JSON");
            black_box(deserialized);
        })
    });
    
    group.finish();
}

fn benchmark_key_operations(c: &mut Criterion) {
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
    let key_bytes = keypair.to_bytes();
    let public_key_bytes = keypair.public_key().to_bytes();
    
    let mut group = c.benchmark_group("key_operations");
    
    group.bench_function("keypair_to_bytes", |b| {
        b.iter(|| {
            let bytes = keypair.to_bytes();
            black_box(bytes);
        })
    });
    
    group.bench_function("keypair_from_bytes", |b| {
        b.iter(|| {
            let restored = KeyPair::from_bytes(&key_bytes)
                .expect("Failed to restore keypair from bytes");
            black_box(restored);
        })
    });
    
    group.bench_function("public_key_to_bytes", |b| {
        b.iter(|| {
            let bytes = keypair.public_key().to_bytes();
            black_box(bytes);
        })
    });
    
    group.bench_function("public_key_from_bytes", |b| {
        b.iter(|| {
            let restored = PublicKey::from_bytes(&public_key_bytes)
                .expect("Failed to restore public key from bytes");
            black_box(restored);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_keypair_generation,
    benchmark_message_signing,
    benchmark_signature_verification,
    benchmark_proof_creation,
    benchmark_proof_verification,
    benchmark_message_creation,
    benchmark_message_verification,
    benchmark_serialization,
    benchmark_key_operations
);

criterion_main!(benches);