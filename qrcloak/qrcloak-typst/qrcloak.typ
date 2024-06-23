#let _sha256plugin = plugin("typst_sha256.wasm")

#let sha256sum(data) = {
  str(_sha256plugin.sha256sum(bytes(json.encode(data, pretty: false))))
} 


#let display = sys.inputs.at("display", default: "true") == "true"

#let qrcloak(path, data, keys, cmd: none, ..args) = {
  keys = (keys,).flatten()

  let meta = (path: path, data: data, cmd: cmd, keys: keys)
  let current = sha256sum(meta)

  if (display) {
    let old = read(path + ".sha256")

    if (old != current) {
      panic("input for path: " + path + " has changed, please run the preprocessor again")
    }
  }

  let img = if (display) { image(path, ..args) } else { none }
  img + [ #metadata(meta) <qrcloak> ]
}
