// Dear ImGui: standalone example application for OSX + Metal.

// Learn about Dear ImGui:
// - FAQ                  https://dearimgui.com/faq
// - Getting Started      https://dearimgui.com/getting-started
// - Documentation        https://dearimgui.com/docs (same as your local docs/ folder).
// - Introduction, links and more at the top of imgui.cpp

#import <Foundation/Foundation.h>

#if TARGET_OS_OSX
#import <Cocoa/Cocoa.h>
#import <AppKit/AppKit.h>
#else
#import <UIKit/UIKit.h>
#endif

#import <Metal/Metal.h>
#import <MetalKit/MetalKit.h>

#include "imgui.h"
#include "imgui_impl_metal.h"
#if TARGET_OS_OSX
#include "imgui_impl_osx.h"
@interface AppViewController : NSViewController<NSWindowDelegate>
@end
#else
@interface AppViewController : UIViewController
@end
#endif

#include <luandabridge.h>

@interface AppViewController () <MTKViewDelegate>
@property (nonatomic, readonly) MTKView *mtkView;
@property (nonatomic, strong) id <MTLDevice> device;
@property (nonatomic, strong) id <MTLCommandQueue> commandQueue;
@property (nonatomic) ImGuiContext *context;
@property (nonatomic) void* rustRenderer;
@end

//-----------------------------------------------------------------------------------
// AppViewController
//-----------------------------------------------------------------------------------

@implementation AppViewController

-(instancetype)initWithNibName:(nullable NSString *)nibNameOrNil bundle:(nullable NSBundle *)nibBundleOrNil
{
    self = [super initWithNibName:nibNameOrNil bundle:nibBundleOrNil];

    _device = MTLCreateSystemDefaultDevice();
    _commandQueue = [_device newCommandQueue];

    _rustRenderer = luanda_renderer_create(_device);

    if (!self.device)
    {
        NSLog(@"Metal is not supported");
        abort();
    }

    // Setup Dear ImGui context
    IMGUI_CHECKVERSION();
    self.context = ImGui::CreateContext();
    ImGuiIO& io = ImGui::GetIO(); (void)io;
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;     // Enable Keyboard Controls
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableGamepad;      // Enable Gamepad Controls
    //io.ConfigFlags |= ImGuiConfigFlags_ViewportsEnable; // Enable Multi-Viewports
    io.ConfigFlags |= ImGuiConfigFlags_DockingEnable;

    // Setup Dear ImGui style
    ImGui::StyleColorsDark();
    //ImGui::StyleColorsLight();

    // Setup Renderer backend
    ImGui_ImplMetal_Init(_device);

    return self;
}

-(void)dealloc
{
    luanda_renderer_destroy(self.rustRenderer);

    [super dealloc];
    ImGui_ImplMetal_Shutdown();
#if TARGET_OS_OSX
    ImGui_ImplOSX_Shutdown();
#endif
    if (self.context)
    {
        NSLog(@"Cleaning up Dear ImGui context.");
        ImGui::DestroyContext(self.context);
        self.context = nullptr;
    }
}

-(MTKView *)mtkView
{
    return (MTKView *)self.view;
}

-(void)loadView
{
    self.view = [[MTKView alloc] initWithFrame:CGRectMake(0, 0, 1200, 800)];
}

-(void)viewDidLoad
{
    [super viewDidLoad];

    self.mtkView.device = self.device;
    self.mtkView.delegate = self;

#if TARGET_OS_OSX
    ImGui_ImplOSX_Init(self.view);
    [NSApp activateIgnoringOtherApps:YES];
#endif
}

-(void)drawInMTKView:(MTKView*)view
{
    ImGuiIO& io = ImGui::GetIO();
    io.DisplaySize.x = view.bounds.size.width;
    io.DisplaySize.y = view.bounds.size.height;

#if TARGET_OS_OSX
    CGFloat framebufferScale = view.window.screen.backingScaleFactor ?: NSScreen.mainScreen.backingScaleFactor;
#else
    CGFloat framebufferScale = view.window.screen.scale ?: UIScreen.mainScreen.scale;
#endif
    io.DisplayFramebufferScale = ImVec2(framebufferScale, framebufferScale);

    id<MTLCommandBuffer> commandBuffer = [self.commandQueue commandBuffer];

    MTLRenderPassDescriptor* renderPassDescriptor = view.currentRenderPassDescriptor;
    if (renderPassDescriptor == nil)
    {
        [commandBuffer commit];
        return;
    }

    // Start the Dear ImGui frame
    ImGui_ImplMetal_NewFrame(renderPassDescriptor);
#if TARGET_OS_OSX
    ImGui_ImplOSX_NewFrame(view);
#endif
    ImGui::NewFrame();

    // Our state (make them static = more or less global) as a convenience to keep the example terse.
    static bool show_demo_window = true;
    static bool show_another_window = false;
    static ImVec4 clear_color = ImVec4(0.45f, 0.55f, 0.60f, 1.00f);

    // Render game to texture
    luanda_renderer_render(self.rustRenderer, (size_t)view.bounds.size.width, (size_t)view.bounds.size.height);
    id<MTLTexture> gameTexture = luanda_renderer_get_texture(self.rustRenderer);

    ImGui::DockSpaceOverViewport(0, ImGui::GetMainViewport());

    // 1. Show the game viewport window with the rendered texture
    ImGui::SetNextWindowSize({400,400});
    ImGui::Begin("Game Viewport");
    if (gameTexture != nil) {
        ImVec2 viewportSize = ImGui::GetContentRegionAvail();
        // Display the texture in the ImGui window
        ImGui::Image((ImTextureID)gameTexture, viewportSize);
    }
    ImGui::End();

    ImGui::SetNextWindowSize({400, ImGui::GetWindowHeight()});
    ImGui::Begin("Scene");
    ImGui::End();

    // Rendering
    ImGui::Render();
    ImDrawData* draw_data = ImGui::GetDrawData();

    if (io.ConfigFlags & ImGuiConfigFlags_ViewportsEnable) {
        ImGui::UpdatePlatformWindows();
        ImGui::RenderPlatformWindowsDefault();
    }

    renderPassDescriptor.colorAttachments[0].clearColor = MTLClearColorMake(clear_color.x * clear_color.w, clear_color.y * clear_color.w, clear_color.z * clear_color.w, clear_color.w);
    id <MTLRenderCommandEncoder> renderEncoder = [commandBuffer renderCommandEncoderWithDescriptor:renderPassDescriptor];
    [renderEncoder pushDebugGroup:@"Dear ImGui rendering"];
    ImGui_ImplMetal_RenderDrawData(draw_data, commandBuffer, renderEncoder);
    [renderEncoder popDebugGroup];
    [renderEncoder endEncoding];

    // Present
    [commandBuffer presentDrawable:view.currentDrawable];
    [commandBuffer commit];
}

-(void)mtkView:(MTKView*)view drawableSizeWillChange:(CGSize)size
{
}

//-----------------------------------------------------------------------------------
// Input processing
//-----------------------------------------------------------------------------------

#if TARGET_OS_OSX

- (void)viewWillAppear
{
    [super viewWillAppear];
    self.view.window.delegate = self;
}

- (void)windowWillClose:(NSNotification *)notification
{
    ImGui_ImplMetal_Shutdown();
    ImGui_ImplOSX_Shutdown();
    ImGui::DestroyContext();
}

#else

// This touch mapping is super cheesy/hacky. We treat any touch on the screen
// as if it were a depressed left mouse button, and we don't bother handling
// multitouch correctly at all. This causes the "cursor" to behave very erratically
// when there are multiple active touches. But for demo purposes, single-touch
// interaction actually works surprisingly well.
-(void)updateIOWithTouchEvent:(UIEvent *)event
{
    UITouch *anyTouch = event.allTouches.anyObject;
    CGPoint touchLocation = [anyTouch locationInView:self.view];
    ImGuiIO &io = ImGui::GetIO();
    io.AddMouseSourceEvent(ImGuiMouseSource_TouchScreen);
    io.AddMousePosEvent(touchLocation.x, touchLocation.y);

    BOOL hasActiveTouch = NO;
    for (UITouch *touch in event.allTouches)
    {
        if (touch.phase != UITouchPhaseEnded && touch.phase != UITouchPhaseCancelled)
        {
            hasActiveTouch = YES;
            break;
        }
    }
    io.AddMouseButtonEvent(0, hasActiveTouch);
}

-(void)touchesBegan:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event      { [self updateIOWithTouchEvent:event]; }
-(void)touchesMoved:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event      { [self updateIOWithTouchEvent:event]; }
-(void)touchesCancelled:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event  { [self updateIOWithTouchEvent:event]; }
-(void)touchesEnded:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event      { [self updateIOWithTouchEvent:event]; }

#endif

@end

//-----------------------------------------------------------------------------------
// AppDelegate
//-----------------------------------------------------------------------------------

#if TARGET_OS_OSX

@interface AppDelegate : NSObject <NSApplicationDelegate, NSToolbarDelegate>
@property (nonatomic, strong) NSWindow *window;
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    NSLog(@"Application finished launching");

    // Create the menu bar
    NSMenu *menuBar = [[NSMenu alloc] init];

    // Create the Application menu (first menu with app name)
    NSMenuItem *appMenuItem = [[NSMenuItem alloc] init];
    NSMenu *appMenu = [[NSMenu alloc] init];
    [appMenu addItemWithTitle:@"Quit" action:@selector(terminate:) keyEquivalent:@"q"];
    [appMenuItem setSubmenu:appMenu];
    [menuBar addItem:appMenuItem];

    // Create File menu
    NSMenuItem *fileMenuItem = [[NSMenuItem alloc] init];
    NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
    [fileMenu addItemWithTitle:@"Open Project" action:@selector(performCustomAction) keyEquivalent:@"O"];
    [fileMenu addItemWithTitle:@"New Project" action:@selector(performCustomAction) keyEquivalent:@"N"];
    [fileMenuItem setSubmenu:fileMenu];
    [menuBar addItem:fileMenuItem];

    // Set the menu bar
    [NSApp setMainMenu:menuBar];
}

- (void)performCustomAction {
    NSLog(@"Custom menu action performed!");
    // Implement your custom logic here
}

#pragma mark - Toolbar Delegate

// Toolbar item identifiers
static NSString *const PlayToolbarItemID = @"PlayItem";
static NSString *const StopToolbarItemID = @"StopItem";
static NSString *const BuildToolbarItemID = @"BuildItem";

- (NSArray<NSToolbarItemIdentifier> *)toolbarAllowedItemIdentifiers:(NSToolbar *)toolbar {
    return @[PlayToolbarItemID,
             StopToolbarItemID,
             BuildToolbarItemID,
             NSToolbarFlexibleSpaceItemIdentifier,
             NSToolbarSpaceItemIdentifier];
}

- (NSArray<NSToolbarItemIdentifier> *)toolbarDefaultItemIdentifiers:(NSToolbar *)toolbar {
    return @[PlayToolbarItemID,
             StopToolbarItemID,
             NSToolbarFlexibleSpaceItemIdentifier,
             BuildToolbarItemID];
}

- (NSToolbarItem *)toolbar:(NSToolbar *)toolbar
     itemForItemIdentifier:(NSToolbarItemIdentifier)itemIdentifier
 willBeInsertedIntoToolbar:(BOOL)flag {

    if ([itemIdentifier isEqualToString:PlayToolbarItemID]) {
        NSToolbarItem *item = [[NSToolbarItem alloc] initWithItemIdentifier:itemIdentifier];
        item.label = @"Play";
        item.paletteLabel = @"Play";
        item.toolTip = @"Run the game";

        if (@available(macOS 11.0, *)) {
            item.image = [NSImage imageWithSystemSymbolName:@"play"
                                   accessibilityDescription:@"Play"];
        } else {
            item.image = [NSImage imageNamed:NSImageNameQuickLookTemplate];
        }

        item.target = self;
        item.action = @selector(playAction:);
        return item;
    }

    if ([itemIdentifier isEqualToString:StopToolbarItemID]) {
        NSToolbarItem *item = [[NSToolbarItem alloc] initWithItemIdentifier:itemIdentifier];
        item.label = @"Stop";
        item.paletteLabel = @"Stop";
        item.toolTip = @"Stop the game";

        if (@available(macOS 11.0, *)) {
            item.image = [NSImage imageWithSystemSymbolName:@"stop"
                                   accessibilityDescription:@"Stop"];
        } else {
            item.image = [NSImage imageNamed:NSImageNameStopProgressTemplate];
        }

        item.target = self;
        item.action = @selector(stopAction:);
        return item;
    }

    if ([itemIdentifier isEqualToString:BuildToolbarItemID]) {
        NSToolbarItem *item = [[NSToolbarItem alloc] initWithItemIdentifier:itemIdentifier];
        item.label = @"Build";
        item.paletteLabel = @"Build";
        item.toolTip = @"Build the project";

        if (@available(macOS 11.0, *)) {
            item.image = [NSImage imageWithSystemSymbolName:@"hammer"
                                   accessibilityDescription:@"Build"];
        } else {
            item.image = [NSImage imageNamed:NSImageNameActionTemplate];
        }

        item.target = self;
        item.action = @selector(buildAction:);
        return item;
    }

    return nil;
}

#pragma mark - Toolbar Actions

- (void)playAction:(id)sender {
    NSLog(@"Play button clicked - starting game");
    // TODO: Implement game play logic
}

- (void)stopAction:(id)sender {
    NSLog(@"Stop button clicked - stopping game");
    // TODO: Implement game stop logic
}

- (void)buildAction:(id)sender {
    NSLog(@"Build button clicked - building project");
    // TODO: Implement build logic
}

-(BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender
{
    return YES;
}

-(instancetype)init
{
    if (self = [super init])
    {
        NSViewController *rootViewController = [[AppViewController alloc] initWithNibName:nil bundle:nil];
        self.window = [[NSWindow alloc] initWithContentRect:NSZeroRect
                                                  styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable
                                                    backing:NSBackingStoreBuffered
                                                      defer:NO];

        // Create and configure toolbar
        NSToolbar *toolbar = [[NSToolbar alloc] initWithIdentifier:@"MainToolbar"];
        toolbar.delegate = self;
        toolbar.displayMode = NSToolbarDisplayModeIconAndLabel;
        toolbar.allowsUserCustomization = YES;
        toolbar.autosavesConfiguration = YES;

        // Use unified toolbar style (macOS 11+)
        if (@available(macOS 11.0, *)) {
            self.window.toolbarStyle = NSWindowToolbarStyleUnified;
        }

        self.window.toolbar = toolbar;
        self.window.contentViewController = rootViewController;
        [self.window center];
        [self.window makeKeyAndOrderFront:self];
    }
    return self;
}

@end

#else

@interface AppDelegate : UIResponder <UIApplicationDelegate>
@property (strong, nonatomic) UIWindow *window;
@end

@implementation AppDelegate

-(BOOL)application:(UIApplication *)application
    didFinishLaunchingWithOptions:(NSDictionary<UIApplicationLaunchOptionsKey,id> *)launchOptions
{
    UIViewController *rootViewController = [[AppViewController alloc] init];
    self.window = [[UIWindow alloc] initWithFrame:UIScreen.mainScreen.bounds];
    self.window.rootViewController = rootViewController;
    [self.window makeKeyAndVisible];
    return YES;
}

@end

#endif

//-----------------------------------------------------------------------------------
// Application main() function
//-----------------------------------------------------------------------------------

#if TARGET_OS_OSX

int main(int, const char**)
{
    @autoreleasepool
    {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];

        AppDelegate *appDelegate = [[AppDelegate alloc] init];   // creates window
        [NSApp setDelegate:appDelegate];

        [NSApp activateIgnoringOtherApps:YES];
        [NSApp run];
    }
    return 0;
}

#else

int main(int argc, char * argv[])
{
    @autoreleasepool
    {
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([AppDelegate class]));
    }
}

#endif
