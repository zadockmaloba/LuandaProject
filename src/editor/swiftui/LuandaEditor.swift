import SwiftUI

struct ContentView: View {
    var body: some View {
        Text("Luanda Editor")
            .frame(minWidth: 800, minHeight: 600)
    }
}

@main
struct LuandaEditor: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

#Preview { ContentView() }
