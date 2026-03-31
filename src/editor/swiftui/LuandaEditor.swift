import SwiftUI

struct SettingsView: View {
    var body: some View {
        Text("Settings")
            .frame(minWidth: 400, minHeight: 300)
    }
}

struct InspectorView: View {
    var body: some View {
        Text("Inspector")
            .frame(minWidth: 300, minHeight: 400)
    }
}

struct AssetsView: View {
    var body: some View {
        List {
            Section(header: Text("2D Meshes")) {
                ForEach(["Line", "Square", "Circle"], id: \.self) { mesh in
                    // add a thumbnail for the mesh
                    Text(mesh)
                }
            }
            Section(header: Text("3D Meshes")) {
                ForEach(["Plane", "Cube", "Sphere"], id: \.self) { mesh in
                    // add a thumbnail for the mesh
                    Text(mesh)
                }
            }
            Section(header: Text("Textures")) {
                ForEach(["Texture1", "Texture2", "Texture3"], id: \.self) { texture in
                    Text(texture)
                }
            }
            Section(header: Text("Models")) {
                ForEach(["Model1", "Model2", "Model3"], id: \.self) { model in
                    Text(model)
                }
            }
            Section(header: Text("Audio")) {
                ForEach(["Audio1", "Audio2", "Audio3"], id: \.self) { audio in
                    Text(audio)
                }
            }.navigationTitle("Assets")
        }
        .frame(minWidth: 300, minHeight: 400)
    }
}

struct SideBarView: View {
    var body: some View {
        TabView {
            AssetsView()
                .tabItem {
                    Label("Assets", systemImage: "photo.on.rectangle")
                        .labelStyle(.titleAndIcon)
                }
            Text("Scene")
                .tabItem {
                    Label("Scene", systemImage: "cube")
                        .labelStyle(.titleAndIcon)
                }
            Text("Hierarchy")
                .tabItem {
                    Label("Hierarchy", systemImage: "list.bullet")
                        .labelStyle(.titleAndIcon)
                }
            Text("Inspector")
                .tabItem {
                    Label("Inspector", systemImage: "slider.horizontal.3")
                        .labelStyle(.titleAndIcon)
                }
        }
        .frame(minWidth: 200)
    }
}

struct ContentView: View {
    @State private var showInspector = false
    var body: some View {
        NavigationSplitView {
            SideBarView()
        } detail: {
            MetalView(color: .black)
        }.inspector(isPresented: $showInspector, content: { InspectorView() })
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
