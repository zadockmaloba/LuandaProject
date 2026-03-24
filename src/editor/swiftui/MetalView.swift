import LuandaBridge
// REF: https://medium.com/@giikwebdeveloper/metal-view-for-swiftui-93f5f78ec36a
import MetalKit
import SwiftUI

// The coordinator class implements the MTKViewDelegate protocol and an instance of
// it is used as the delegate for the wrapped MTKView.
class MetalViewCoordinator: NSObject, MTKViewDelegate {
  let device: MTLDevice
  // The Metal Programming Guide recommends that non-transient objects
  // including command queues are reused, especially in performance-sensitive
  // code.
  let commandQueue: MTLCommandQueue
  var rustRenderer: UnsafeMutableRawPointer? = nil

  override init() {
    guard let device = MTLCreateSystemDefaultDevice(),
      let queue = device.makeCommandQueue()
    else {
      fatalError("could not create device or command queue")
    }
    // Technically, we don't need to save the device in the coordinator. The
    // MTKView must have its device property set so this is an easy way to
    // ensure that the devices used by the coordinator and the view are
    // identical.
    self.device = device
    self.commandQueue = queue
    self.rustRenderer = luanda_renderer_create(device)
    super.init()
  }

  deinit {
    luanda_renderer_destroy(self.rustRenderer)
  }

  func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

  // Most importantly, the delegate implements the draw(in:) method which we don't call
  // directly. Rather, we request for it to be called by calling the draw() method of
  // the wrapped MTKView.
  func draw(in view: MTKView) {
    luanda_renderer_draw(
      self.rustRenderer,
      view.currentRenderPassDescriptor
    )
  }
}

struct MetalView: NSViewRepresentable {
  @Environment(\.self) var environment  // needed to resolve Color instances
  var color: Color  // allows us to inject a clear color

  func makeCoordinator() -> MetalViewCoordinator {
    MetalViewCoordinator()
  }

  // Creating the wrapped MTKView
  func makeNSView(context: Context) -> MTKView {
    let view = MTKView()
    view.delegate = context.coordinator
    view.device = context.coordinator.device
    print("made the MTKView")
    return view
  }

  // Called when the view needs to be updated, incl. because the value of the
  // Binding changed. It gives us a chance to call draw() which in turn will
  // call draw(in:) on the delegate.
  func updateNSView(_ view: MTKView, context: Context) {
    let resolved = self.color.resolve(in: self.environment)
    view.clearColor = MTLClearColor(
      red: Double(resolved.red),
      green: Double(resolved.green),
      blue: Double(resolved.blue),
      alpha: Double(resolved.opacity)
    )
    view.draw()
  }
}

#Preview {
  MetalView(color: Color.red)
}
