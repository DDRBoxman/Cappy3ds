//
//  ContentView.swift
//  Cappy3ds
//
//  Created by Colin Edwards on 9/25/23.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundColor(.accentColor)
            Text("Hello, world!")
            RenderView()
        }
        .padding()
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}


struct RenderView: NSViewRepresentable {


    func makeNSView(context: Context) -> NSView {
        let tmp = NSView()
        hello_world()
        
      
        NSViewBridge.send(tmp)

        return tmp
    }

    func updateNSView(_ view: NSView, context: Context) {

    }
}

