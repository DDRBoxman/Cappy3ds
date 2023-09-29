using Windows.UI.Composition;
using Microsoft.UI.Xaml.Hosting;
using System;
using System.Diagnostics;
using System.Numerics;
using WinRT;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Composition;

namespace Cappy3ds
{
    internal class RendewView: Grid
    {

       // IntPtr idcomp_ref;
       // Microsoft.UI.Composition.Visual? MUXVisual;
        //Microsoft.UI.Content.ContentExternalOutputLink contentExternalOutputLink;

        public async void setup()
        {


            /*Compositor compositor = ElementCompositionPreview.GetElementVisual(this).Compositor;

            contentExternalOutputLink = Microsoft.UI.Content.ContentExternalOutputLink.Create(compositor);

            contentExternalOutputLink.BackgroundColor = Windows.UI.Color.FromArgb(255, 54, 192, 255);*/

            //contentExternalOutputLink.tryAs<CompositionTarget>();

            /*var WUXTarget = CompositionTarget.FromAbi(
                ((IWinRTObject)contentExternalOutputLink).NativeObject.GetRef()
            );*/
            //contentExternalOutputLink.NativeObject.GetRef();
            // idcomp_ref = ((IWinRTObject)contentExternalOutputLink).NativeObject.GetRef();

            // var WUXTarget = CompositionTarget.FromAbi(
            // contentExternalOutputLink
            //   );

            // var ptr = contentExternalOutputLink as CompositionTarget;

            /* var placementVisual = contentExternalOutputLink.PlacementVisual;
             placementVisual.Size = new Vector2(400, 400);

             ElementCompositionPreview.SetElementChildVisual(this, placementVisual);

             //var target = ContentExternalOutputLink.As<CompositionTarget>();
             //EnsureWindowsSystemDispatcherQueueController();

             //var WUXCompositor = new Compositor();
             //var compositionTarget = compositor.Create();

             Console.WriteLine("Hello World!");
             Debug.Write("Hello, This is written in the Debug window");

             var WUXTarget = Windows.UI.Composition.CompositionTarget.FromAbi(
                 ((IWinRTObject)contentExternalOutputLink).NativeObject.GetRef()
             );*/

            /*var test = new WebView2Ex();

            var (MUXVisual, WUXVisual, pointer) = await test.CreateVisual();
            //visual ??= WUXVisual;
            this.MUXVisual = MUXVisual;

           // SetCoreWebViewAndVisualSize((float)ActualWidth, (float)ActualHeight);

            ElementCompositionPreview.SetElementChildVisual(this, MUXVisual);

            unsafe
            {
                CsBindgen.NativeMethods.send_visual((void*)pointer);
            }
            */
            //SetElementChildVisual(this, placementVisual);


           // _root = _compositor.CreateContainerVisual();



            // _compositionTarget = _compositor.CreateTargetForCurrentView();
            // _compositionTarget.Root = _root;

            //
            // Create a few visuals for our window
            //
            //for (int index = 0; index < 20; index++)
            // {
            // _root.Children.InsertAtTop(CreateChildElement());
            // }



            //WUXTarget.Root = WUXVisual;

            // await MUXCompositor.RequestCommitAsync();
            // await WUXCompositor.RequestCommitAsync();

            /* winrt::com_ptr<IDCompositionTarget> target = m_outputLink.as< IDCompositionTarget > ();

             HRESULT hr = target->SetRoot(m_hostedVisual.get());
             assert(SUCCEEDED(hr));

             winrt::float2 size{ 400, 300};
             m_outputLink.PlacementVisual().Size(size);
             winrt::ElementCompositionPreview::SetElementChildVisual(*this, m_outputLink.PlacementVisual());

             // Hard-coding the size for now since during early init we don't seem to always
             // have an initial size yet in WinUI 3.
             SetHostedVisualSize(400, 301); // ISSUE: Why can't this be exactly 300 when window is cloaked?*/

        }

    }


}


