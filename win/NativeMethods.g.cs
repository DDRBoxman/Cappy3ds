// <auto-generated>
// This code is generated by csbindgen.
// DON'T CHANGE THIS DIRECTLY.
// </auto-generated>
#pragma warning disable CS8500
#pragma warning disable CS8981
using System;
using System.Runtime.InteropServices;


namespace CsBindgen
{
    internal static unsafe partial class NativeMethods
    {
        const string __DllName = "cappy3ds_render";



        [DllImport(__DllName, EntryPoint = "send_swap_chain_panel", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void send_swap_chain_panel(void* swap_chain_panel);

        [DllImport(__DllName, EntryPoint = "send_window", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void send_window(void* app_kit_nsview);


    }



}
    