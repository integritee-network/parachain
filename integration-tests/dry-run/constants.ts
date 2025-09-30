import {
    XcmV5Junction,
    XcmV5Junctions,
    XcmV5NetworkId,
} from "@polkadot-api/descriptors";
import {Binary, FixedSizeBinary} from "polkadot-api";
import {AccountId} from "@polkadot-api/substrate-bindings";

export const KAH_PARA_ID = 1000;
export const PAH_PARA_ID = 1000;
export const IK_PARA_ID = 2015;
export const IP_PARA_ID = 2039;

export const TEER_UNITS = 1_000_000_000_000n;
export const KSM_UNITS = 1_000_000_000_000n;
export const DOT_UNITS = 10_000_000_000n;

export const KAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(KAH_PARA_ID)),
};
export const PAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(PAH_PARA_ID)),
};
export const KAH_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(KAH_PARA_ID)]),
};
export const PAH_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(PAH_PARA_ID)]),
};
export const KSM_FROM_COUSIN_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama())),
};
export const KSM_FROM_SIBLING_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
export const DOT_FROM_COUSIN_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot())),
};
export const DOT_FROM_SIBLING_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
export const TEER_FROM_SELF = {
    parents: 0,
    interior: XcmV5Junctions.Here(),
};
export const ITK_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
};
export const ITK_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(IK_PARA_ID)]),
};
export const ITP_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IP_PARA_ID)),
};
export const ITP_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(IP_PARA_ID)]),
};

const palletId = Buffer.from("modlpy/trsry", "utf8"); // 8 bytes
const padded = Buffer.concat([palletId, Buffer.alloc(32 - palletId.length, 0)]);
const treasuryAccount = FixedSizeBinary.fromHex(padded.toHex());

export const TREASURY_LOCAL = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: treasuryAccount}))
}
export const TREASURY_PAH = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: Binary.fromBytes(AccountId().enc("14xmwinmCEz6oRrFdczHKqHgWNMiCysE2KrA4jXXAAM1Eogk"))}))
}
export const TREASURY_KAH = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: Binary.fromBytes(AccountId().enc("HWZmQq6zMMk7TxixHfseFT2ewicT6UofPa68VCn3gkXrdJF"))}))
}

export const ALICE_PUB = FixedSizeBinary.fromHex("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"); // well-known alice account
export const ALICE_LOCAL = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: ALICE_PUB}))
}