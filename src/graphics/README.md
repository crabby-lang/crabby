# Graphics Integrated support for Crabby

By using rust crates such as `ash` for **Vulkan**, and `wgpu` for OpenGL, **Crabby** is now
fully supporting and integrating Native Graphics code.

## Example

Here is a Crabby `code snippets` for painting:

```py
def paint() {
    canvas(800, 600)
    background("white")

    brush("red")
    draw(50, 50, 100, 100)

    circle(200, 200, 50)
    line(0, 0, 800, 600)

    render() // Shows the output
}
```
