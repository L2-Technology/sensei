// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

pub mod adaptor;
pub mod admin;
pub mod node;
pub mod utils;

pub mod sensei {
    use senseicore::node::{
        LocalInvoice, LocalInvoiceFeatures, LocalRouteHint, LocalRouteHintHop, LocalRoutingFees,
    };

    tonic::include_proto!("sensei");

    impl From<LocalInvoice> for Invoice {
        fn from(invoice: LocalInvoice) -> Self {
            Invoice {
                payment_hash: invoice.payment_hash,
                currency: invoice.currency,
                amount: invoice.amount,
                description: invoice.description,
                expiry: invoice.expiry,
                timestamp: invoice.timestamp,
                min_final_cltv_expiry: invoice.min_final_cltv_expiry,
                route_hints: invoice
                    .route_hints
                    .into_iter()
                    .map(|h| LocalRouteHint::from(&h).into())
                    .collect(),
                features: invoice.features.map(|f| f.into()),
                payee_pub_key: invoice.payee_pub_key.to_string(),
            }
        }
    }

    impl From<LocalRouteHint> for RouteHint {
        fn from(hint: LocalRouteHint) -> Self {
            Self {
                hops: hint
                    .hops
                    .into_iter()
                    .map(|h| LocalRouteHintHop::from(&h).into())
                    .collect(),
            }
        }
    }

    impl From<LocalRouteHintHop> for RouteHintHop {
        fn from(hop: LocalRouteHintHop) -> Self {
            Self {
                src_node_id: hop.src_node_id.to_string(),
                short_channel_id: hop.short_channel_id,
                fees: Some(LocalRoutingFees::from(hop.fees).into()),
                cltv_expiry_delta: hop.cltv_expiry_delta.into(),
                htlc_minimum_msat: hop.htlc_minimum_msat,
                htlc_maximum_msat: hop.htlc_maximum_msat,
            }
        }
    }

    impl From<LocalRoutingFees> for RoutingFees {
        fn from(fees: LocalRoutingFees) -> Self {
            Self {
                base_msat: fees.base_msat,
                proportional_millionths: fees.proportional_millionths,
            }
        }
    }

    impl From<LocalInvoiceFeatures> for Features {
        fn from(features: LocalInvoiceFeatures) -> Self {
            Self {
                variable_length_onion: features.variable_length_onion,
                payment_secret: features.payment_secret,
                basic_mpp: features.basic_mpp,
            }
        }
    }
}
