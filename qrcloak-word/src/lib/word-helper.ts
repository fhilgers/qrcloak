export function wordReady() {
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

export async function insertImage(b64Image: string) {
  const replaced = b64Image.replace(/^data:image\/png;base64,/, "");
  return await Word.run(async (context) => {
    context.document.body.insertInlinePictureFromBase64(
      replaced,
      Word.InsertLocation.end,
    );
    return await context.sync();
  });
}
