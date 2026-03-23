import SwiftUI

struct SettingsView: View {
    var body: some View {
        Text("Settings")
            .frame(minWidth: 400, minHeight: 300)
    }
}

struct SideBarView: View {
    var body: some View {
        TabView {
            Text("Hierarchy")
                .tabItem {
                    Label("Hierarchy", systemImage: "list.bullet")
                }
            Text("Scene")
                .tabItem {
                    Label("Scene", systemImage: "cube")
                }
            Text("Assets")
                .tabItem {
                    Label("Assets", systemImage: "photo.on.rectangle")
                }
            Text("Inspector")
                .tabItem {
                    Label("Inspector", systemImage: "slider.horizontal.3")
                }
        }
        .frame(minWidth: 200)
    }
}

struct ContentView: View {
    var body: some View {
        NavigationSplitView {
            SideBarView()
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
