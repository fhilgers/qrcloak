import { createEffect, createSignal } from "solid-js";
import { encrypt_to_b64png } from "qrypt-wasm";

function wordReady() {
  return new Promise<void>((resolve, reject) => {
    Office.onReady((info) => {
      if (info.host === Office.HostType.Word) {
        resolve();
      } else {
        reject(info);
      }
    });
  });
}

export const TaskPane = (props: {}) => {
  const [password, setPassword] = createSignal("");
  const [text, setText] = createSignal("");
  const [ready, setReady] = createSignal(false);

  createEffect(() => {
    wordReady()
      .then(() => setReady(true))
      .catch(console.error);
  });

  return (
    <>
      <div class="flex flex-col h-screen space-y-4 p-4">
        <textarea
          class="flex-1 border border-[#79747E] rounded-md p-2 bg-[#FEF7FF] text-[#1D1B20]"
          placeholder="Data to Encrypt..."
          onInput={(e) => setText(e.target.value)}
        >
          {text()}
        </textarea>

        <input
          type="password"
          class="border border-[#79747E] rounded-md p-2 bg-[#FEF7FF] text-[#1D1B20]"
          placeholder="Password"
          onInput={(e) => setPassword(e.target.value)}
        >
          {password()}
        </input>

        <button
          class="bg-blue-500 disabled:bg-gray-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-full h-10"
          disabled={text() == "" || password() == "" || !ready()}
          onClick={async () => await encryptAndInsert(text(), password())}
        >
          Encrypt
        </button>
      </div>
    </>
  );
};

async function encryptAndInsert(text: string, password: string) {
  const encryptedData = encrypt_to_b64png(text, password);

  return Word.run(async (context) => {
    context.document.body.insertInlinePictureFromBase64(
      encryptedData,
      Word.InsertLocation.end,
    );

    return await context.sync();
  });
}
