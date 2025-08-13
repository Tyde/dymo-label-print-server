/*#let grocery = sys.inputs.at("grocery", default: "Weizenmehl")
#let subtitle = sys.inputs.at("subtitle", default: "Type 550")*/

#let file-path = "data.yml"
#let data = yaml(file-path)
#let grocery = data.grocery
#let subtitle = data.subtitle
#let small-title = data.small-title
#let title-text-size = if (small-title == true) {
    30pt
} else {
    39pt
}

#set page(width:89mm, height:36mm, margin:0mm)

#set text(size:title-text-size, font:"Pleasewritemeasong")
#align(center + horizon)[
    #par(leading:8pt)[
    #grocery\
    #text(size:14pt)[#subtitle]]
]