import { Show, createEffect, createSignal } from "solid-js";

import "@material/web/textfield/outlined-text-field";
import "@material/web/select/outlined-select";
import "@material/web/select/select-option";
import "@material/web/slider/slider";
import "@material/web/button/filled-tonal-button";

import "./app.css";

import { wordReady, insertImage } from "~/lib/word-helper";

import {
  AgeRecipient,
  Passphrase,
  PayloadGenerator,
  Encryption as QrCloakEncryption,
  Compression as QrCloakCompression,
  GzipCompression,
  PayloadEncoder,
  Payloads,
  PayloadSplitter,
} from "qrcloak";

const EncryptionOptions = [
  "No Encryption",
  "Age Passphrase",
  "Age Key",
] as const;
type Encryption = (typeof EncryptionOptions)[number];
const encryptionOptions = EncryptionOptions.map((e) => e);

const CompressionOptions = ["No Compression", "Gzip"] as const;
type Compression = (typeof CompressionOptions)[number];
const compressionOptions = CompressionOptions.map((e) => e);

function encryptionToQrCloakEncryption(
  encryption: Encryption,
  secret: string,
): QrCloakEncryption {
  switch (encryption) {
    case "No Encryption":
      return "NoEncryption";
    case "Age Passphrase":
      return { AgePassphrase: { passphrase: new Passphrase(secret) } };
    case "Age Key":
      return { AgeKey: { recipients: [new AgeRecipient(secret)] } };
  }
}

function compressionToQrCloakCompression(
  compression: Compression,
): QrCloakCompression {
  switch (compression) {
    case "No Compression":
      return "NoCompression";
    case "Gzip":
      return { Gzip: { gzip: new GzipCompression() } };
  }
}

function createPayloads(
  text: string,
  secret: string,
  splits: number,
  encryption: Encryption,
  compression: Compression,
): Payloads {
  const qrCloakEncryptoin = encryptionToQrCloakEncryption(encryption, secret);
  const qrCloakCompression = compressionToQrCloakCompression(compression);

  const payload = new PayloadGenerator()
    .with_encryption(qrCloakEncryptoin)
    .with_compression(qrCloakCompression)
    .generate(text);

  if (splits == 0) {
    return [payload];
  }

  return new PayloadSplitter().with_splits(splits).split(payload);
}

function encodePayloads(payloads: Payloads): string[] {
  const encoded = new PayloadEncoder()
    .with_encoding({ Json: { pretty: false, merge: false } })
    .encode(payloads);

  return encoded;
}

import { toDataURL } from "qrcode";

async function toSvg(encoded: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    toDataURL(encoded, function (err, url) {
      if (err) {
        reject(err);
      }
      resolve(url);
    });
  });
}

export default function App() {
  const [text, setText] = createSignal("");
  const [secret, setSecret] = createSignal("");

  const [currentEncryption, setCurrentEncryption] =
    createSignal<Encryption>("No Encryption");

  const [currentCompression, setCurrentCompression] =
    createSignal<Compression>("No Compression");

  const visible = () => {
    if (currentEncryption() === "No Encryption") {
      return false;
    }
    return true;
  };

  const [splits, setSplits] = createSignal(0);

  const [encodedPayloads, setEncodedPayloads] = createSignal<string[]>([]);

  const [imageSrcs, setImageSrcs] = createSignal<string[]>([]);

  const [ready, setReady] = createSignal(false);

  createEffect(() => {
    console.log("waiting");
    wordReady()
      .then(() => {
        console.log("ready");
        setReady(true);
      })
      .catch(console.error);
  });

  createEffect(async () => {
    let newSrcs = await Promise.all(
      encodedPayloads().map(async (encodedPayload) => {
        return await toSvg(encodedPayload);
      }),
    );

    setImageSrcs(newSrcs);
  });

  return (
    <main>
      <div class="flex flex-col items-center w-full h-full min-h-screen p-8 gap-8 bg-slate-50">
        <md-outlined-text-field
          class="w-full max-w-screen-sm"
          prop:type="textarea"
          prop:value={text()}
          onChange={(e) => setText(e.currentTarget.value)}
        ></md-outlined-text-field>

        <div class="flex flex-row w-full max-w-screen-sm gap-8 justify-between">
          <md-outlined-select
            class="flex-1"
            prop:label="Encryption"
            prop:value={currentEncryption()}
            onChange={(e) =>
              setCurrentEncryption(e.currentTarget.value as Encryption)
            }
          >
            {encryptionOptions.map((e) => (
              <md-select-option prop:value={e}>{e}</md-select-option>
            ))}
          </md-outlined-select>

          <md-outlined-select
            class="flex-1"
            prop:label="Compression"
            prop:value={currentCompression()}
            onChange={(e) =>
              setCurrentCompression(e.currentTarget.value as Compression)
            }
          >
            {compressionOptions.map((e) => (
              <md-select-option prop:value={e}>{e}</md-select-option>
            ))}
          </md-outlined-select>
        </div>

        <md-slider
          class="w-full max-w-screen-sm"
          prop:ticks={true}
          prop:step={1}
          prop:min={0}
          prop:max={8}
          prop:labeled={true}
          prop:value={splits()}
          onChange={(e) => setSplits(e.currentTarget.value as number)}
        ></md-slider>

        <Show when={visible()}>
          <md-outlined-text-field
            class="w-full max-w-screen-sm"
            prop:type="password"
            prop:value={secret()}
            onChange={(e) => setSecret(e.currentTarget.value)}
          ></md-outlined-text-field>
        </Show>

        <md-filled-tonal-button
          class="w-full max-w-screen-sm"
          onClick={() => {
            let payloads = createPayloads(
              text(),
              secret(),
              splits(),
              currentEncryption(),
              currentCompression(),
            );
            let encoded = encodePayloads(payloads);
            setEncodedPayloads(encoded);
          }}
        >
          Generate
        </md-filled-tonal-button>

        <div class="w-full max-w-screen-sm">
          <div class="grid grid-cols-2 gap-8 w-full">
            {imageSrcs().map((src, i) => (
              <div class="flex flex-col items-center">
                <md-filled-tonal-button
                  onClick={async () => await insertImage(src)}
                >
                  Insert
                </md-filled-tonal-button>
                <img class="w-full" src={src} alt="QR Code" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </main>
  );
}
