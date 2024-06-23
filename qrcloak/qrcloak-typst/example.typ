#import "qrcloak.typ": qrcloak

= Simple Showcase


#let hello = "Hello"
#let cmdData = "cat my_secret.txt"
#let ageKeys = ("age1hqfc0jzpezgpjlnr2mkc2uy3hq5eklyx5yxc95ynae4cdajcydqq7vk4ta")

#grid(
  columns: (1fr, 1fr), gutter: 24pt, align(start)[
    This is very sensitive information:

    #qrcloak("qrcodes/qrcode1.png", hello, ageKeys, width: 100%)
  ], align(
    start,
  )[
    This is a secret read from another file:

    #qrcloak("qrcodes/qrcode2.png", cmdData, ageKeys, cmd: "bash", width: 100%)
  ],
)

