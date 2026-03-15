#include "luandaeditor.h"
#include <QVBoxLayout>
#include <QDockWidget>

struct LuandaEditor::LuandaEditorPrivate {
    LuandaEditorPrivate() {
    }
};

LuandaEditor::LuandaEditor(QWidget *parent)
    : QMainWindow(parent) {
}

LuandaEditor::~LuandaEditor() {
}

auto LuandaEditor::addDockWidgetHelper(Qt::DockWidgetArea area, QWidget *widget) -> void {
    auto dock_widget = new QDockWidget(this);
    dock_widget->setWidget(widget);

    this->addDockWidget(area, std::move(dock_widget));
}
