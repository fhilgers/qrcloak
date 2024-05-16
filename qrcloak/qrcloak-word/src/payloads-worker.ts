import {
  Encryption,
  Compression,
  createPayloads,
  encodePayloads,
} from "~/payloads";

export type MessageData = {
  text: string;
  secret: string;
  splits: number;
  encryption: Encryption;
  compression: Compression;
};

export function handle(e: MessageData) {
  let payloads = createPayloads(
    e.text,
    e.secret,
    e.splits,
    e.encryption,
    e.compression,
  );

  let encoded = encodePayloads(payloads);

  return encoded;
}

//onmessage = (e: MessageEvent<MessageData>) => {
//  postMessage(handle(e.data));
//};
