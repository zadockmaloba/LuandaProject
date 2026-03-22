#include "luandaeditor.h"
#include "explorerwidget.h"
#include "viewportwidget.h"

#include <QApplication>

auto main(int argc, char *argv[]) -> int {
    QApplication a(argc, argv);
    auto w = LuandaEditor();
    w.setBaseSize(800,600);
    w.setCentralWidget(new ViewPortWidget);
    w.addDockWidgetHelper(Qt::LeftDockWidgetArea, new ExplorerWidget);
    w.show();
    return QCoreApplication::exec();
}
