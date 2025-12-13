use hyperliquid_core::types::*;
use serde_json::{json, Value};

#[test]
fn test_meta_struct_serialization_deserialization() {
    // Create a sample Meta struct
    let meta = Meta {
        universe: vec![
            AssetMeta {
                name: "BTC".to_string(),
                onlyIsolated: false,
                szDecimals: 3,
                maxLeverage: 50,
                maxDynamicLeverage: Some(100),
                type_: Some("perpetual".to_string()),
                tokens: None,
                maxOi: Some("1000000".to_string()),
                underlying: Some("BTC".to_string()),
                isInverse: Some(false),
            },
            AssetMeta {
                name: "ETH".to_string(),
                onlyIsolated: true,
                szDecimals: 2,
                maxLeverage: 25,
                maxDynamicLeverage: None,
                type_: Some("perpetual".to_string()),
                tokens: None,
                maxOi: Some("500000".to_string()),
                underlying: Some("ETH".to_string()),
                isInverse: Some(false),
            },
        ],
        exchange: Some(ExchangeMeta {
            vaults: vec![
                VaultMeta {
                    vault: "vault1".to_string(),
                    name: "Test Vault 1".to_string(),
                    creator: "0x1234".to_string(),
                    creatorLong: "Long Creator 1".to_string(),
                    creatorShort: "Short Creator 1".to_string(),
                    price: Some("1000.50".to_string()),
                },
                VaultMeta {
                    vault: "vault2".to_string(),
                    name: "Test Vault 2".to_string(),
                    creator: "0x5678".to_string(),
                    creatorLong: "Long Creator 2".to_string(),
                    creatorShort: "Short Creator 2".to_string(),
                    price: None,
                },
            ],
        }),
    };

    // Serialize to JSON
    let json_string = serde_json::to_string(&meta).expect("Failed to serialize Meta");
    println!("Serialized JSON: {}", json_string);

    // Deserialize back from JSON
    let deserialized: Meta = serde_json::from_str(&json_string)
        .expect("Failed to deserialize Meta");

    // Verify roundtrip equality
    assert_eq!(meta.universe.len(), deserialized.universe.len());
    assert_eq!(meta.universe[0].name, deserialized.universe[0].name);
    assert_eq!(meta.universe[0].onlyIsolated, deserialized.universe[0].onlyIsolated);
    assert_eq!(meta.universe[0].szDecimals, deserialized.universe[0].szDecimals);
    assert_eq!(meta.universe[0].maxLeverage, deserialized.universe[0].maxLeverage);
    assert_eq!(meta.universe[0].maxDynamicLeverage, deserialized.universe[0].maxDynamicLeverage);
    assert_eq!(meta.universe[0].type_, deserialized.universe[0].type_);
    assert_eq!(meta.universe[0].maxOi, deserialized.universe[0].maxOi);
    assert_eq!(meta.universe[0].underlying, deserialized.universe[0].underlying);
    assert_eq!(meta.universe[0].isInverse, deserialized.universe[0].isInverse);

    // Verify exchange metadata
    assert!(meta.exchange.is_some());
    assert!(deserialized.exchange.is_some());

    let exchange_meta = meta.exchange.as_ref().unwrap();
    let deserialized_exchange = deserialized.exchange.as_ref().unwrap();

    assert_eq!(exchange_meta.vaults.len(), deserialized_exchange.vaults.len());
    assert_eq!(exchange_meta.vaults[0].vault, deserialized_exchange.vaults[0].vault);
    assert_eq!(exchange_meta.vaults[0].name, deserialized_exchange.vaults[0].name);
    assert_eq!(exchange_meta.vaults[0].creator, deserialized_exchange.vaults[0].creator);
    assert_eq!(exchange_meta.vaults[0].price, deserialized_exchange.vaults[0].price);

    println!("✅ Meta struct roundtrip serialization/deserialization test passed");
}

#[test]
fn test_meta_from_json() {
    // Sample JSON response from Hyperliquid API
    let json_data = json!({
        "universe": [
            {
                "name": "BTC",
                "onlyIsolated": false,
                "szDecimals": 3,
                "maxLeverage": 50,
                "maxDynamicLeverage": 100,
                "type": "perpetual",
                "tokens": null,
                "maxOi": "1000000",
                "underlying": "BTC",
                "isInverse": false
            },
            {
                "name": "ETH",
                "onlyIsolated": true,
                "szDecimals": 2,
                "maxLeverage": 25,
                "maxDynamicLeverage": null,
                "type": "perpetual",
                "tokens": null,
                "maxOi": "500000",
                "underlying": "ETH",
                "isInverse": false
            }
        ],
        "exchange": {
            "vaults": [
                {
                    "vault": "vault1",
                    "name": "Test Vault 1",
                    "creator": "0x1234",
                    "creatorLong": "Long Creator 1",
                    "creatorShort": "Short Creator 1",
                    "price": "1000.50"
                },
                {
                    "vault": "vault2",
                    "name": "Test Vault 2",
                    "creator": "0x5678",
                    "creatorLong": "Long Creator 2",
                    "creatorShort": "Short Creator 2",
                    "price": null
                }
            ]
        }
    });

    // Parse JSON into Meta struct
    let meta: Meta = serde_json::from_value(json_data.clone())
        .expect("Failed to parse Meta from JSON");

    // Verify all fields are parsed correctly
    assert_eq!(meta.universe.len(), 2);

    // Verify first asset
    let btc_asset = &meta.universe[0];
    assert_eq!(btc_asset.name, "BTC");
    assert_eq!(btc_asset.onlyIsolated, false);
    assert_eq!(btc_asset.szDecimals, 3);
    assert_eq!(btc_asset.maxLeverage, 50);
    assert_eq!(btc_asset.maxDynamicLeverage, Some(100));
    assert_eq!(btc_asset.type_, Some("perpetual".to_string()));
    assert_eq!(btc_asset.tokens, None);
    assert_eq!(btc_asset.maxOi, Some("1000000".to_string()));
    assert_eq!(btc_asset.underlying, Some("BTC".to_string()));
    assert_eq!(btc_asset.isInverse, Some(false));

    // Verify second asset
    let eth_asset = &meta.universe[1];
    assert_eq!(eth_asset.name, "ETH");
    assert_eq!(eth_asset.onlyIsolated, true);
    assert_eq!(eth_asset.szDecimals, 2);
    assert_eq!(eth_asset.maxLeverage, 25);
    assert_eq!(eth_asset.maxDynamicLeverage, None);
    assert_eq!(eth_asset.type_, Some("perpetual".to_string()));
    assert_eq!(eth_asset.tokens, None);
    assert_eq!(eth_asset.maxOi, Some("500000".to_string()));
    assert_eq!(eth_asset.underlying, Some("ETH".to_string()));
    assert_eq!(eth_asset.isInverse, Some(false));

    // Verify exchange metadata
    assert!(meta.exchange.is_some());
    let exchange = meta.exchange.as_ref().unwrap();
    assert_eq!(exchange.vaults.len(), 2);

    let vault1 = &exchange.vaults[0];
    assert_eq!(vault1.vault, "vault1");
    assert_eq!(vault1.name, "Test Vault 1");
    assert_eq!(vault1.creator, "0x1234");
    assert_eq!(vault1.creatorLong, "Long Creator 1");
    assert_eq!(vault1.creatorShort, "Short Creator 1");
    assert_eq!(vault1.price, Some("1000.50".to_string()));

    let vault2 = &exchange.vaults[1];
    assert_eq!(vault2.vault, "vault2");
    assert_eq!(vault2.name, "Test Vault 2");
    assert_eq!(vault2.creator, "0x5678");
    assert_eq!(vault2.creatorLong, "Long Creator 2");
    assert_eq!(vault2.creatorShort, "Short Creator 2");
    assert_eq!(vault2.price, None);

    println!("✅ Meta struct from JSON test passed");
}

#[test]
fn test_meta_serialize_back_to_json() {
    // Create Meta struct
    let meta = Meta {
        universe: vec![
            AssetMeta {
                name: "SOL".to_string(),
                onlyIsolated: false,
                szDecimals: 1,
                maxLeverage: 20,
                maxDynamicLeverage: Some(40),
                type_: Some("perpetual".to_string()),
                tokens: None,
                maxOi: Some("200000".to_string()),
                underlying: Some("SOL".to_string()),
                isInverse: Some(false),
            },
        ],
        exchange: Some(ExchangeMeta {
            vaults: vec![VaultMeta {
                vault: "sol_vault".to_string(),
                name: "SOL Vault".to_string(),
                creator: "0xabcd".to_string(),
                creatorLong: "SOL Long Creator".to_string(),
                creatorShort: "SOL Short Creator".to_string(),
                price: Some("150.75".to_string()),
            }],
        }),
    };

    // Serialize to JSON
    let json_value: Value = serde_json::to_value(&meta)
        .expect("Failed to serialize Meta to JSON value");

    // Verify JSON structure
    assert!(json_value.is_object());
    let obj = json_value.as_object().unwrap();

    // Verify universe array
    let universe = obj.get("universe").expect("Missing 'universe' field");
    assert!(universe.is_array());
    let universe_array = universe.as_array().unwrap();
    assert_eq!(universe_array.len(), 1);

    // Verify first asset in universe
    let sol_asset = &universe_array[0];
    assert_eq!(sol_asset.get("name").and_then(|v| v.as_str()), Some("SOL"));
    assert_eq!(sol_asset.get("onlyIsolated").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(sol_asset.get("szDecimals").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(sol_asset.get("maxLeverage").and_then(|v| v.as_i64()), Some(20));
    assert_eq!(sol_asset.get("maxDynamicLeverage").and_then(|v| v.as_i64()), Some(40));
    assert_eq!(sol_asset.get("type").and_then(|v| v.as_str()), Some("perpetual"));
    assert_eq!(sol_asset.get("maxOi").and_then(|v| v.as_str()), Some("200000"));
    assert_eq!(sol_asset.get("underlying").and_then(|v| v.as_str()), Some("SOL"));
    assert_eq!(sol_asset.get("isInverse").and_then(|v| v.as_bool()), Some(false));

    // Verify exchange field
    let exchange = obj.get("exchange").expect("Missing 'exchange' field");
    assert!(exchange.is_object());

    let exchange_obj = exchange.as_object().unwrap();
    let vaults = exchange_obj.get("vaults").expect("Missing 'vaults' field");
    assert!(vaults.is_array());

    let vaults_array = vaults.as_array().unwrap();
    assert_eq!(vaults_array.len(), 1);

    let vault = &vaults_array[0];
    assert_eq!(vault.get("vault").and_then(|v| v.as_str()), Some("sol_vault"));
    assert_eq!(vault.get("name").and_then(|v| v.as_str()), Some("SOL Vault"));
    assert_eq!(vault.get("creator").and_then(|v| v.as_str()), Some("0xabcd"));
    assert_eq!(vault.get("price").and_then(|v| v.as_str()), Some("150.75"));

    println!("✅ Meta struct serialize back to JSON test passed");
}

#[test]
fn test_meta_with_null_exchange() {
    // Test Meta with null exchange (some API responses may not include exchange)
    let json_data = json!({
        "universe": [
            {
                "name": "BTC",
                "onlyIsolated": false,
                "szDecimals": 3,
                "maxLeverage": 50
            }
        ],
        "exchange": null
    });

    let meta: Meta = serde_json::from_value(json_data)
        .expect("Failed to parse Meta with null exchange");

    assert_eq!(meta.universe.len(), 1);
    assert_eq!(meta.universe[0].name, "BTC");
    assert!(meta.exchange.is_none());

    println!("✅ Meta struct with null exchange test passed");
}

#[test]
fn test_meta_with_missing_exchange() {
    // Test Meta without exchange field (should default to None)
    let json_data = json!({
        "universe": [
            {
                "name": "ETH",
                "onlyIsolated": true,
                "szDecimals": 2,
                "maxLeverage": 25
            }
        ]
    });

    let meta: Meta = serde_json::from_value(json_data)
        .expect("Failed to parse Meta without exchange field");

    assert_eq!(meta.universe.len(), 1);
    assert_eq!(meta.universe[0].name, "ETH");
    assert!(meta.exchange.is_none());

    println!("✅ Meta struct without exchange field test passed");
}

#[test]
fn test_asset_meta_optional_fields() {
    // Test AssetMeta with various combinations of optional fields
    let test_cases = vec![
        (
            json!({
                "name": "BTC",
                "onlyIsolated": false,
                "szDecimals": 3,
                "maxLeverage": 50
            }),
            AssetMeta {
                name: "BTC".to_string(),
                onlyIsolated: false,
                szDecimals: 3,
                maxLeverage: 50,
                maxDynamicLeverage: None,
                type_: None,
                tokens: None,
                maxOi: None,
                underlying: None,
                isInverse: None,
            },
        ),
        (
            json!({
                "name": "ETH",
                "onlyIsolated": true,
                "szDecimals": 2,
                "maxLeverage": 25,
                "maxDynamicLeverage": 50,
                "type": "perpetual",
                "maxOi": "500000",
                "underlying": "ETH",
                "isInverse": false
            }),
            AssetMeta {
                name: "ETH".to_string(),
                onlyIsolated: true,
                szDecimals: 2,
                maxLeverage: 25,
                maxDynamicLeverage: Some(50),
                type_: Some("perpetual".to_string()),
                tokens: None,
                maxOi: Some("500000".to_string()),
                underlying: Some("ETH".to_string()),
                isInverse: Some(false),
            },
        ),
    ];

    for (json_input, expected) in test_cases {
        let asset: AssetMeta = serde_json::from_value(json_input.clone())
            .expect("Failed to parse AssetMeta");

        assert_eq!(asset.name, expected.name);
        assert_eq!(asset.onlyIsolated, expected.onlyIsolated);
        assert_eq!(asset.szDecimals, expected.szDecimals);
        assert_eq!(asset.maxLeverage, expected.maxLeverage);
        assert_eq!(asset.maxDynamicLeverage, expected.maxDynamicLeverage);
        assert_eq!(asset.type_, expected.type_);
        assert_eq!(asset.maxOi, expected.maxOi);
        assert_eq!(asset.underlying, expected.underlying);
        assert_eq!(asset.isInverse, expected.isInverse);

        // Roundtrip test
        let serialized = serde_json::to_value(&asset)
            .expect("Failed to serialize AssetMeta");
        let roundtrip: AssetMeta = serde_json::from_value(serialized)
            .expect("Failed to deserialize AssetMeta");

        assert_eq!(roundtrip.name, expected.name);
        assert_eq!(roundtrip.onlyIsolated, expected.onlyIsolated);
        assert_eq!(roundtrip.szDecimals, expected.szDecimals);
        assert_eq!(roundtrip.maxLeverage, expected.maxLeverage);
        assert_eq!(roundtrip.maxDynamicLeverage, expected.maxDynamicLeverage);
        assert_eq!(roundtrip.type_, expected.type_);
        assert_eq!(roundtrip.maxOi, expected.maxOi);
        assert_eq!(roundtrip.underlying, expected.underlying);
        assert_eq!(roundtrip.isInverse, expected.isInverse);
    }

    println!("✅ AssetMeta optional fields test passed");
}

#[test]
fn test_candle_ohlcv_data_parsing() {
    // Test parsing candleSnapshot response
    let json_data = json!({
        "coin": "BTC",
        "interval": "1h",
        "start": 1704067200000,
        "end": 1704070800000,
        "trades": 150,
        "txHash": "0xabc123",
        "open": "50000.50",
        "high": "51000.00",
        "low": "49500.25",
        "close": "50500.75",
        "volume": "100.5",
        "vwap": "50250.10",
        "bidVolume": "50.25",
        "bidVwap": "50100.20",
        "askVolume": "50.25",
        "askVwap": "50400.30"
    });

    let candle: Candle = serde_json::from_value(json_data.clone())
        .expect("Failed to parse Candle from JSON");

    // Verify OHLCV fields
    assert_eq!(candle.coin, "BTC");
    assert_eq!(candle.interval, "1h");
    assert_eq!(candle.start, 1704067200000);
    assert_eq!(candle.end, 1704070800000);
    assert_eq!(candle.trades, Some(150));
    assert_eq!(candle.txHash, Some("0xabc123".to_string()));
    assert_eq!(candle.open, "50000.50");
    assert_eq!(candle.high, "51000.00");
    assert_eq!(candle.low, "49500.25");
    assert_eq!(candle.close, "50500.75");
    assert_eq!(candle.volume, "100.5");
    assert_eq!(candle.vwap, "50250.10");
    assert_eq!(candle.bidVolume, Some("50.25".to_string()));
    assert_eq!(candle.bidVwap, Some("50100.20".to_string()));
    assert_eq!(candle.askVolume, Some("50.25".to_string()));
    assert_eq!(candle.askVwap, Some("50400.30".to_string()));

    println!("✅ Candle OHLCV data parsing test passed");
}

#[test]
fn test_candle_roundtrip_serialization() {
    // Create Candle struct
    let candle = Candle {
        coin: "ETH".to_string(),
        interval: "5m".to_string(),
        start: 1704067200000,
        end: 1704067500000,
        trades: Some(25),
        txHash: Some("0xdef456".to_string()),
        open: "3000.00".to_string(),
        close: "3010.50".to_string(),
        high: "3020.00".to_string(),
        low: "2990.00".to_string(),
        volume: "50.25".to_string(),
        vwap: "3005.25".to_string(),
        bidVolume: Some("25.10".to_string()),
        bidVwap: Some("3002.50".to_string()),
        askVolume: Some("25.15".to_string()),
        askVwap: Some("3008.00".to_string()),
    };

    // Serialize to JSON
    let json_string = serde_json::to_string(&candle)
        .expect("Failed to serialize Candle");
    println!("Serialized Candle JSON: {}", json_string);

    // Deserialize back from JSON
    let deserialized: Candle = serde_json::from_str(&json_string)
        .expect("Failed to deserialize Candle");

    // Verify roundtrip equality
    assert_eq!(candle.coin, deserialized.coin);
    assert_eq!(candle.interval, deserialized.interval);
    assert_eq!(candle.start, deserialized.start);
    assert_eq!(candle.end, deserialized.end);
    assert_eq!(candle.trades, deserialized.trades);
    assert_eq!(candle.txHash, deserialized.txHash);
    assert_eq!(candle.open, deserialized.open);
    assert_eq!(candle.high, deserialized.high);
    assert_eq!(candle.low, deserialized.low);
    assert_eq!(candle.close, deserialized.close);
    assert_eq!(candle.volume, deserialized.volume);
    assert_eq!(candle.vwap, deserialized.vwap);
    assert_eq!(candle.bidVolume, deserialized.bidVolume);
    assert_eq!(candle.bidVwap, deserialized.bidVwap);
    assert_eq!(candle.askVolume, deserialized.askVolume);
    assert_eq!(candle.askVwap, deserialized.askVwap);

    println!("✅ Candle roundtrip serialization test passed");
}

#[test]
fn test_candle_with_optional_fields_null() {
    // Test Candle with some optional fields as null
    let json_data = json!({
        "coin": "SOL",
        "interval": "15m",
        "start": 1704067200000,
        "end": 1704068100000,
        "trades": null,
        "txHash": null,
        "open": "150.00",
        "high": "155.00",
        "low": "148.00",
        "close": "152.50",
        "volume": "1000.0",
        "vwap": "151.75",
        "bidVolume": null,
        "bidVwap": null,
        "askVolume": null,
        "askVwap": null
    });

    let candle: Candle = serde_json::from_value(json_data)
        .expect("Failed to parse Candle with null fields");

    assert_eq!(candle.coin, "SOL");
    assert_eq!(candle.interval, "15m");
    assert_eq!(candle.start, 1704067200000);
    assert_eq!(candle.end, 1704068100000);
    assert_eq!(candle.trades, None);
    assert_eq!(candle.txHash, None);
    assert_eq!(candle.open, "150.00");
    assert_eq!(candle.high, "155.00");
    assert_eq!(candle.low, "148.00");
    assert_eq!(candle.close, "152.50");
    assert_eq!(candle.volume, "1000.0");
    assert_eq!(candle.vwap, "151.75");
    assert_eq!(candle.bidVolume, None);
    assert_eq!(candle.bidVwap, None);
    assert_eq!(candle.askVolume, None);
    assert_eq!(candle.askVwap, None);

    println!("✅ Candle with optional fields null test passed");
}

#[test]
fn test_candle_timestamp_parsing() {
    // Test timestamp parsing from milliseconds
    let json_data = json!({
        "coin": "BTC",
        "interval": "1d",
        "start": 1704067200000,  // 2024-01-01 00:00:00 UTC
        "end": 1704153600000,    // 2024-01-02 00:00:00 UTC
        "open": "50000.00",
        "high": "52000.00",
        "low": "48000.00",
        "close": "51000.00",
        "volume": "1000.0",
        "vwap": "50500.00"
    });

    let candle: Candle = serde_json::from_value(json_data)
        .expect("Failed to parse Candle timestamp");

    // Verify timestamp values
    assert_eq!(candle.start, 1704067200000);
    assert_eq!(candle.end, 1704153600000);

    // Convert to human-readable format for verification
    // Note: In a real implementation, you might want to use chrono for this
    assert!(candle.start > 0);
    assert!(candle.end > candle.start);

    println!("✅ Candle timestamp parsing test passed");
}

#[test]
fn test_candle_interval_enum() {
    // Test various interval values that are commonly used
    let intervals = vec![
        "1m", "5m", "15m", "30m",
        "1h", "2h", "4h", "6h", "12h",
        "1d", "3d", "7d", "14d",
        "1w", "2w", "1M", "3M"
    ];

    for interval in intervals {
        let json_data = json!({
            "coin": "BTC",
            "interval": interval,
            "start": 1704067200000,
            "end": 1704070800000,
            "open": "50000.00",
            "high": "51000.00",
            "low": "49000.00",
            "close": "50500.00",
            "volume": "100.0",
            "vwap": "50250.00"
        });

        let candle: Candle = serde_json::from_value(json_data)
            .expect(&format!("Failed to parse Candle with interval: {}", interval));

        assert_eq!(candle.interval, interval);
    }

    println!("✅ Candle interval enum test passed");
}

#[test]
fn test_candle_websocket_message() {
    // Test Candle WebSocket message parsing
    let json_data = json!({
        "channel": "candle.BTC",
        "data": {
            "coin": "BTC",
            "interval": "5m",
            "start": 1704067200000,
            "end": 1704067500000,
            "open": "50000.00",
            "high": "50100.00",
            "low": "49900.00",
            "close": "50050.00",
            "volume": "50.0",
            "vwap": "50025.00"
        },
        "time": 1704067500000
    });

    let ws_msg: WsMsg = serde_json::from_value(json_data)
        .expect("Failed to parse Candle WebSocket message");

    match ws_msg {
        WsMsg::CandleMsg(candle_msg) => {
            assert_eq!(candle_msg.data.coin, "BTC");
            assert_eq!(candle_msg.data.interval, "5m");
            assert_eq!(candle_msg.data.start, 1704067200000);
            assert_eq!(candle_msg.data.end, 1704067500000);
            assert_eq!(candle_msg.data.open, "50000.00");
            assert_eq!(candle_msg.data.high, "50100.00");
            assert_eq!(candle_msg.data.low, "49900.00");
            assert_eq!(candle_msg.data.close, "50050.00");
            assert_eq!(candle_msg.data.volume, "50.0");
            assert_eq!(candle_msg.data.vwap, "50025.00");
        }
        _ => panic!("Expected CandleMsg variant"),
    }

    println!("✅ Candle WebSocket message test passed");
}

#[test]
fn test_subscription_type_variants() {
    // Test allMids subscription
    let all_mids = Subscription::AllMids;
    let all_mids_json = serde_json::to_value(&all_mids).unwrap();
    let expected_all_mids = json!({"type": "allMids"});
    assert_eq!(all_mids_json, expected_all_mids);

    let parsed_all_mids: Subscription = serde_json::from_value(expected_all_mids).unwrap();
    assert_eq!(parsed_all_mids, all_mids);

    // Test l2Book subscription
    let l2_book = Subscription::L2Book { coin: "BTC".to_string() };
    let l2_book_json = serde_json::to_value(&l2_book).unwrap();
    let expected_l2_book = json!({"type": "l2Book", "coin": "BTC"});
    assert_eq!(l2_book_json, expected_l2_book);

    let parsed_l2_book: Subscription = serde_json::from_value(expected_l2_book).unwrap();
    assert_eq!(parsed_l2_book, l2_book);

    // Test trades subscription
    let trades = Subscription::Trades { coin: "ETH".to_string() };
    let trades_json = serde_json::to_value(&trades).unwrap();
    let expected_trades = json!({"type": "trades", "coin": "ETH"});
    assert_eq!(trades_json, expected_trades);

    let parsed_trades: Subscription = serde_json::from_value(expected_trades).unwrap();
    assert_eq!(parsed_trades, trades);

    // Test bbo subscription
    let bbo = Subscription::Bbo { coin: "SOL".to_string() };
    let bbo_json = serde_json::to_value(&bbo).unwrap();
    let expected_bbo = json!({"type": "bbo", "coin": "SOL"});
    assert_eq!(bbo_json, expected_bbo);

    let parsed_bbo: Subscription = serde_json::from_value(expected_bbo).unwrap();
    assert_eq!(parsed_bbo, bbo);

    // Test candle subscription
    let candle = Subscription::Candle { coin: "BTC".to_string(), interval: "1m".to_string() };
    let candle_json = serde_json::to_value(&candle).unwrap();
    let expected_candle = json!({"type": "candle", "coin": "BTC", "interval": "1m"});
    assert_eq!(candle_json, expected_candle);

    let parsed_candle: Subscription = serde_json::from_value(expected_candle).unwrap();
    assert_eq!(parsed_candle, candle);

    // Test userEvents subscription
    let user_events = Subscription::UserEvents { user: "0x1234567890abcdef".to_string() };
    let user_events_json = serde_json::to_value(&user_events).unwrap();
    let expected_user_events = json!({"type": "userEvents", "user": "0x1234567890abcdef"});
    assert_eq!(user_events_json, expected_user_events);

    let parsed_user_events: Subscription = serde_json::from_value(expected_user_events).unwrap();
    assert_eq!(parsed_user_events, user_events);

    // Test userFills subscription
    let user_fills = Subscription::UserFills { user: "0xabcdef1234567890".to_string() };
    let user_fills_json = serde_json::to_value(&user_fills).unwrap();
    let expected_user_fills = json!({"type": "userFills", "user": "0xabcdef1234567890"});
    assert_eq!(user_fills_json, expected_user_fills);

    let parsed_user_fills: Subscription = serde_json::from_value(expected_user_fills).unwrap();
    assert_eq!(parsed_user_fills, user_fills);

    // Test orderUpdates subscription
    let order_updates = Subscription::OrderUpdates { user: "0xfedcba0987654321".to_string() };
    let order_updates_json = serde_json::to_value(&order_updates).unwrap();
    let expected_order_updates = json!({"type": "orderUpdates", "user": "0xfedcba0987654321"});
    assert_eq!(order_updates_json, expected_order_updates);

    let parsed_order_updates: Subscription = serde_json::from_value(expected_order_updates).unwrap();
    assert_eq!(parsed_order_updates, order_updates);

    // Test userFundings subscription
    let user_fundings = Subscription::UserFundings { user: "0x1111111111111111".to_string() };
    let user_fundings_json = serde_json::to_value(&user_fundings).unwrap();
    let expected_user_fundings = json!({"type": "userFundings", "user": "0x1111111111111111"});
    assert_eq!(user_fundings_json, expected_user_fundings);

    let parsed_user_fundings: Subscription = serde_json::from_value(expected_user_fundings).unwrap();
    assert_eq!(parsed_user_fundings, user_fundings);

    // Test userNonFundingLedgerUpdates subscription
    let user_ledger = Subscription::UserNonFundingLedgerUpdates { user: "0x2222222222222222".to_string() };
    let user_ledger_json = serde_json::to_value(&user_ledger).unwrap();
    let expected_user_ledger = json!({"type": "userNonFundingLedgerUpdates", "user": "0x2222222222222222"});
    assert_eq!(user_ledger_json, expected_user_ledger);

    let parsed_user_ledger: Subscription = serde_json::from_value(expected_user_ledger).unwrap();
    assert_eq!(parsed_user_ledger, user_ledger);

    // Test webData2 subscription
    let web_data = Subscription::WebData2 { user: "0x3333333333333333".to_string() };
    let web_data_json = serde_json::to_value(&web_data).unwrap();
    let expected_web_data = json!({"type": "webData2", "user": "0x3333333333333333"});
    assert_eq!(web_data_json, expected_web_data);

    let parsed_web_data: Subscription = serde_json::from_value(expected_web_data).unwrap();
    assert_eq!(parsed_web_data, web_data);

    // Test activeAssetCtx subscription
    let active_ctx = Subscription::ActiveAssetCtx { coin: "BTC".to_string() };
    let active_ctx_json = serde_json::to_value(&active_ctx).unwrap();
    let expected_active_ctx = json!({"type": "activeAssetCtx", "coin": "BTC"});
    assert_eq!(active_ctx_json, expected_active_ctx);

    let parsed_active_ctx: Subscription = serde_json::from_value(expected_active_ctx).unwrap();
    assert_eq!(parsed_active_ctx, active_ctx);

    // Test activeAssetData subscription
    let active_data = Subscription::ActiveAssetData {
        user: "0x4444444444444444".to_string(),
        coin: "ETH".to_string()
    };
    let active_data_json = serde_json::to_value(&active_data).unwrap();
    let expected_active_data = json!({"type": "activeAssetData", "user": "0x4444444444444444", "coin": "ETH"});
    assert_eq!(active_data_json, expected_active_data);

    let parsed_active_data: Subscription = serde_json::from_value(expected_active_data).unwrap();
    assert_eq!(parsed_active_data, active_data);

    println!("✅ All subscription type variants test passed");
}