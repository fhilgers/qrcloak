
#let display = sys.inputs.at("display", default: "true") == "true"

#let paths = json.decode(sys.inputs.at("paths", default: "{}"))

#let qrcloak(
    path,
    data,
    keys,
) = {
    if (not paths.at(path, default: false)) and display {
        panic("qrcode for " + path + " not found")
    }

    //assert(false, message: json.encode((keys,)))

    let img = if (display) { image(path) } else { none } 
    img + [ #metadata((path: path, data: data, keys: (keys,).flatten(),)) <qrcloak> ] 
}

#qrcloak("qrcodes/first.png", "This is something", ("age1hqfc0jzpezgpjlnr2mkc2uy3hq5eklyx5yxc95ynae4cdajcydqq7vk4ta",))

#qrcloak("qrcodes/second.png", "This is something
over multiple lines", "age1hqfc0jzpezgpjlnr2mkc2uy3hq5eklyx5yxc95ynae4cdajcydqq7vk4ta")