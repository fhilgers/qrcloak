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

export async function insertImages(b64Images: string[]) {
	const replaced = b64Images.map((i) =>
		i.replace(/^data:image\/png;base64,/, ""),
	);
	return await Word.run(async (context) => {
		replaced.forEach((element) => {
			context.document.body.insertInlinePictureFromBase64(
				element,
				Word.InsertLocation.end,
			);
		});

		return await context.sync();
	});
}
