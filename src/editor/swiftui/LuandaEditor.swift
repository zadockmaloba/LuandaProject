import SwiftUI

struct SettingsView: View {
    var body: some View {
        Text("Settings")
            .frame(minWidth: 400, minHeight: 300)
    }
}

struct ContentView: View {
    var body: some View {
        NavigationSplitView {
            List {
                Text("Item 1")
                Text("Item 2")
                Text("Item 3")
            }
            .listStyle(SidebarListStyle())
        } detail: {
            MetalView(color: .blue)
                .edgesIgnoringSafeArea(.all)
        }
    }
}

@main
struct LuandaEditor: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
        .commands {
            SidebarCommands()
        }
#if os(macOS)
        Settings {
            SettingsView()
        }
#endif
    }
}

#Preview { ContentView() }
