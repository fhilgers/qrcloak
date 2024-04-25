import { Show, createEffect, createSignal } from "solid-js";

import "@material/web/button/elevated-button";
import "@material/web/button/filled-tonal-button";
import "@material/web/button/outlined-button";
import "@material/web/icon/icon";
import "@material/web/progress/circular-progress";
import "@material/web/select/outlined-select";
import "@material/web/select/select-option";
import "@material/web/slider/slider";
import "@material/web/textfield/filled-text-field";
import "@material/web/textfield/outlined-text-field";

import "@fontsource/material-icons-outlined/latin-400.css";
import "@fontsource/roboto/latin-400.css";
import "@fontsource/roboto/latin-500.css";
import "@fontsource/roboto/latin-700.css";

import "~/app.css";

import { insertImages, wordReady } from "~/lib/word-helper";
import {
  Compression,
  Encryption,
  compressionOptions,
  encryptionOptions,
  toSvg,
} from "~/payloads";

import { MessageData, handle } from "~/payloads-worker";

export default function AppInner() {
  //let worker: Worker;

  const [payloadsMessage, setPayloadsMessage] = createSignal<MessageData>();

  createEffect(async () => {
    let message = payloadsMessage();

    if (message) {
      setIsEncoding(true);
      try {
        let result = handle(message);
        setEncodedPayloads(result);
      } catch (e: any) {
        setError(e);
      }

      setIsEncoding(false);
    }

    // TODO: fix age scrypt factor estimation which is very slow in worker
    /*
    if (window.Worker) {
      worker = worker instanceof Worker ? worker : new PayloadsWorker();

      worker.postMessage(payloadsMessage());

      worker.addEventListener("message", (e: MessageEvent<string[]>) => {
        setEncodedPayloads(e.data);
        setIsEncoding(false); 
      });
    }
    */
  });

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

  const [error, setError] = createSignal<string | null>(null);

  const [isEncoding, setIsEncoding] = createSignal(false);

  createEffect(() => {
    console.log("error", error());
  });

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
      <div class="flex flex-col items-center w-full h-full min-h-screen p-8 gap-8 bg-[#FEF7FF]">
        <md-outlined-text-field
          class="w-full max-w-screen-sm"
          prop:type="textarea"
          prop:value={text()}
          prop:label="Text to Encrypt"
          onChange={(e) => {
            setText(e.currentTarget.value);
          }}
        ></md-outlined-text-field>

        <div class="flex flex-col sm:flex-row w-full max-w-screen-sm gap-8 justify-between">
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
            prop:label={currentEncryption()}
            prop:value={secret()}
            prop:error={error() !== null}
            prop:errorText={error() ?? ""}
            onKeyPress={(e) => {
              setError(null);
            }}
            onChange={(e) => {
              setError(null);
              setSecret(e.currentTarget.value);
            }}
          ></md-outlined-text-field>
        </Show>

        <md-filled-tonal-button
          class="w-full max-w-screen-sm h-[56px]"
          prop:disabled={isEncoding()}
          onClick={async () => {
            setPayloadsMessage({
              text: text(),
              secret: secret(),
              splits: splits(),
              encryption: currentEncryption(),
              compression: currentCompression(),
            });
          }}
        >
          {isEncoding() ? (
            <md-circular-progress prop:indeterminate={true}>
              {" "}
            </md-circular-progress>
          ) : (
            "Generate"
          )}
        </md-filled-tonal-button>

        <Show when={imageSrcs().length > 0}>
          <div class="w-full max-w-screen-sm flex flex-col items-start rounded-2xl">
            <md-filled-tonal-button
              onClick={async () => {
                await insertImages(imageSrcs());
              }}
              class="my-4"
            >
              Insert All
            </md-filled-tonal-button>
            <div class="flex flex-col sm:grid sm:grid-cols-2 gap-8 w-full">
              {imageSrcs().map((src, i) => (
                <div class="flex flex-col gap-2 items-end p-4 rounded-2xl bg-[#E6E0E9]">
                  <img
                    class="w-full h-full border-2 rounded-lg"
                    src={src}
                    alt="QR Code"
                  />
                  <div></div>
                  <md-elevated-button
                    onClick={async () => await insertImages([src])}
                  >
                    <md-icon slot="icon">add</md-icon>
                    Insert
                  </md-elevated-button>
                </div>
              ))}
            </div>
          </div>
        </Show>
      </div>
    </main>
  );
}
