---
layout: post
short_title: Using ActivationViewSwitcher
title: Multiple Views in UWP - ActivationViewSwitcher
author: yourordinarycat
---

If you're maintaining a UWP app with multiple windows, you're likely aware of how tricky it is to manage the main view's closure (AKA consolidation). If your user clicks on an "open in new window" button, and accidentally closes the initial window, any subsequent attempts to launch the app will get them booted to the secondary window they opened a bit ago. So they now have to close the app to access the main view again. Not a fun time for anyone involved.

Lucky for you, there is a solution - [`ActivationViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.activationviewswitcher?view=winrt-22621)! It is rather underused from what I could gather, but it works, and fairly well at that. We'll be working with C# and XAML in this post, but the APIs can be used from any language that can work with WinRT. Without further ado, let's dive right in!

## Default Behavior

On activation, if your app is already open, the Windows shell will pick a view to show the user. Chances are, this view will be the last one the user got to interact with. It isn't always so simple though, because those views could be consolidated at any point. Because of this, the process could be more accurately described as follows:

> Out of all candidates, pick the last non-consolidated view the user interacted with.

Looks good so far, but there is a (perhaps surprising) catch. If the main view is consolidated, the shell won't show it again - it'll go for the next view as described above. In a way, the "main view" isn't all that different compared to other views - what makes it special is that all activation events are delivered on its thread, and when the view closes, the app goes down with it. For view switching though, it is just another view... for better or for worse.

So, what now? You could attempt to switch back to the main view through [`ApplicationViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.applicationviewswitcher?view=winrt-22621) on every activation, but this doesn't work very well, because you'll always end up with the main view on screen. To add insult to injury, the shell will still show the view it wanted to show, and if that's the secondary view, it'll end up behind the main one you just presented. In short, don't do that.

## Getting Started

This example will be starting from a simple C# XAML app. The current launch handler looks like this:

```C#
using System;
using Windows.ApplicationModel.Activation;
using Windows.UI.ViewManagement;
using Windows.UI.Xaml;

// ...

protected override void OnLaunched(LaunchActivatedEventArgs args)
{
    var window = Window.Current;
    if (window.Content == null)
    {
        window.Content = new MainPage();
        window.Activate();
    }
}
```

Quite often, apps will have a Frame as the root element for their window, but that doesn't matter much for this example. What _does_ matter for the example is having a method to create new views. This should work well enough:

```C#
using System;
using System.Threading.Tasks;
using Windows.ApplicationModel.Core;
using Windows.UI.ViewManagement;

// ...

public static Task<ApplicationView> CreateViewAsync()
{
    var cav = CoreApplication.CreateNewView();
    var tcs = new TaskCompletionSource<ApplicationView>();

    bool enqueued = cav.DispatcherQueue.TryEnqueue(() =>
    {
        var win = Window.Current;
        win.Content = new SecondaryPage();
        win.Activate();

        tcs.SetResult(ApplicationView.GetForCurrentView());
    });

    if (enqueued)
    {
        return tcs.Task;
    }

    var err = new Exception("Unable to create a new view.");
    return Task.FromException<ApplicationView>(err);
}
```

### Overriding the Shell's Behavior

There is a way to override the shell's behavior - calling [`ApplicationViewSwitcher.DisableSystemViewActivationPolicy()`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.applicationviewswitcher.disablesystemviewactivationpolicy?view=winrt-22621). This will disable the shell's view switching policy during the app's current session - after the app is closed, you'll have to call this again to prevent the shell from taking over. I recommend calling this method during launch, so that you have consistent behavior at all times.

### Manually Handling View Switching

First off, let's create a class to handle view switching. The behavior we're going for is:

> Always show the main view if it has been consolidated. Otherwise, show the latest active view, unless that view has been consolidated - in that case, show the main view.

```C#
using System.Threading.Tasks;
using Windows.ApplicationModel.Core;
using Windows.Foundation;
using Windows.UI.Core;
using Windows.UI.ViewManagement;

// ...

public sealed class ActivationViewManager
{
    private readonly int MainViewId;
    private int LastActivatedViewId;

    private bool MainViewConsolidated = false;
    private bool UseLastActivatedViewId = false;

    private ActivationViewManager(CoreWindow cw, ApplicationView av)
    {
        cw.Activated += OnMainViewActivated;
        av.Consolidated += OnMainViewConsolidated;

        MainViewId = av.Id;
    }

    public static ActivationViewManager Current { get; private set; }

    public static Task<bool> InitializeAsync()
    {
        if (Current != null)
        {
            return Task.FromResult(true);
        }

        var mv = CoreApplication.MainView;
        var tcs = new TaskCompletionSource<bool>();

        bool enqueued = mv.DispatcherQueue.TryEnqueue(() =>
        {
            Current = new ActivationViewManager(mv.CoreWindow, ApplicationView.GetForCurrentView());
            tcs.SetResult(true);
        });

        if (enqueued)
        {
            return tcs.Task;
        }

        return Task.FromResult(false);
    }

    private void OnMainViewActivated(CoreWindow sender, WindowActivatedEventArgs args)
    {
        MainViewConsolidated = false;
        LastActivatedViewId = ApplicationView.GetApplicationViewIdForWindow(sender);
        UseLastActivatedViewId = true;
    }

    private void OnMainViewConsolidated(ApplicationView sender, ApplicationViewConsolidatedEventArgs args)
    {
        MainViewConsolidated = true;
        UseLastActivatedViewId = false;
    }
}
```

Lots to unpack here. Let's start out by going through initialization - the view manager is created within the main view's thread, and adds event handlers for its activation and consolidation. The activation handler updates `MainViewConsolidated` to indicate that the main view is not consolidated, stores the active view's Id, and through `UseLastActivatedViewId`, indicates to the class that the view with the most recently stored Id should be shown when activating the app. The consolidation handler tells the class that the main view is now consolidated, and the most recently stored Id should not be used for now.

### Tracking Secondary Views

Of course, we may not want to show the main view on every activation. To fix this, we can add a way to register secondary views, so that they can be candidates during activation as well:

```C#
public void RegisterCurrentView()
{
    CoreWindow.GetForCurrentThread().Activated += OnRegisteredViewActivated;
    ApplicationView.GetForCurrentView().Consolidated += OnRegisteredViewConsolidated;
}

// ...

private void OnRegisteredViewActivated(CoreWindow sender, WindowActivatedEventArgs args)
{
    LastActivatedViewId = ApplicationView.GetApplicationViewIdForWindow(sender);
    UseLastActivatedViewId = true;
}

private void OnRegisteredViewConsolidated(ApplicationView sender, ApplicationViewConsolidatedEventArgs args)
{
    UseLastActivatedViewId = false;
    CoreWindow.GetForCurrentThread().Close();
}
```

Note that the consolidation handler in this example also closes the view's [`CoreWindow`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.core.corewindow?view=winrt-22621) - that part might not be necessary if you have separate handling for this.

### Handling Activation

Here comes the most exciting part - actually showing the view we want to display in response to activation! The code for this is quite simple:

```C#
public IAsyncAction HandleActivationAsync(ActivationViewSwitcher switcher)
{
    if (MainViewConsolidated || !UseLastActivatedViewId)
    {
        return switcher.ShowAsStandaloneAsync(MainViewId);
    }

    return switcher.ShowAsStandaloneAsync(LastActivatedViewId);
}
```

You don't want any heavy operations here, as the shell will wait for your app to pick its preferred view. Waiting for the start menu to dismiss itself for multiple seconds is not something your users want.

The [`ShowAsStandaloneAsync`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.activationviewswitcher.showasstandaloneasync?view=winrt-22621) method works like the similarly named [`ApplicationViewSwitcher.TryShowAsStandaloneAsync()`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.applicationviewswitcher.tryshowasstandaloneasync?view=winrt-22621). However, there is no result for its operation, because that wouldn't make much sense. Another difference is, this method should not be called more than once from the same instance of [`ActivationViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.activationviewswitcher?view=winrt-22621) - you can't respond to an activation by showing more than a single view.

Now we can wire it all together. Let's ensure that we're registering our secondary views after creating them:

```C#
// ...

ActivationViewManager.Current.RegisterCurrentView();

var win = Window.Current;
win.Content = new SecondaryPage();
win.Activate();

// ...
```

Our `OnLaunched` should now look like this:

```C#
protected override async void OnLaunched(LaunchActivatedEventArgs args)
{
    var window = Window.Current;
    if (window.Content == null)
    {
        ApplicationViewSwitcher.DisableSystemViewActivationPolicy();
        await ActivationViewManager.InitializeAsync();

        window.Content = new MainPage();
        window.Activate();
    }
    else if (args.ViewSwitcher != null)
    {
        await ActivationViewManager.Current.HandleActivationAsync(args.ViewSwitcher);
    }
}
```

### Using the IViewSwitcherProvider Interface

Activation args that provide an [`ActivationViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.activationviewswitcher?view=winrt-22621) will do so through [`IViewSwitcherProvider`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.activation.iviewswitcherprovider?view=winrt-22621), which in turn provides the [`ViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.activation.iviewswitcherprovider.viewswitcher?view=winrt-22621) property. From a generic activation handler, you can cast the args to (or, in COM speak, query for) [`IViewSwitcherProvider`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.activation.iviewswitcherprovider?view=winrt-22621), and check whether its [`ViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.activation.iviewswitcherprovider.viewswitcher?view=winrt-22621) property is non null - if both are the case, you can control activation all by yourself. Keep in mind that any activations you don't handle after calling [`ApplicationViewSwitcher.DisableSystemViewActivationPolicy()`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.applicationviewswitcher.disablesystemviewactivationpolicy?view=winrt-22621) will remain unhandled - it'll look like your app failed to show itself.

In this example, we can check if an [`ActivationViewSwitcher`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.activationviewswitcher?view=winrt-22621) is available directly, because we have concrete activation event args. If you're writing a generic handler, checking for [`IViewSwitcherProvider`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.activation.iviewswitcherprovider?view=winrt-22621) should be a safe bet.

## Wrapping Up

We're done! Now you can make your users happier, and more productive. 🎉

It is a shame that this is so underused. The default behavior (and how to disable it) should've been better documented, but it is quite late for that. If you're dealing with multiple views in UWP, you might want to follow this article! Chances are, it will lead to a better experience, for you and your users.
