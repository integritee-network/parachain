import {Counter, Gauge, register} from "prom-client";
import express from "express";

export const accountBalanceGauge = new Gauge({
    name: "account_balance",
    help: "Balance of an account",
    labelNames: ["name", "address", "chain", "asset"],
});

export const assetSupplyGauge = new Gauge({
    name: "asset_supply",
    help: "total supply of an asset on a chain",
    labelNames: ["chain", "asset"],
});

export const assetConversionGauge = new Gauge({
    name: "asset_conversion_price",
    help: "DEX price of a pair ona a chain",
    labelNames: ["base_asset", "quote_asset", "chain"],
});

const app = express();

export function startPrometheusMetrics(prometheus_port: number) {
    app.get('/metrics', async (req, res) => {
        res.set('Content-Type', register.contentType);
        res.end(await register.metrics());
    });
    app.listen(prometheus_port, () => {
        console.log(`Prometheus metrics available at http://localhost:${prometheus_port}/metrics`);
    });
}

