#include "explorerwidget.h"
#include <QTabWidget>
#include <QTreeWidget>
#include <QTreeView>
#include <QHBoxLayout>
#include <memory>

struct ExplorerWidget::ExplorerWidgetPrivate {
    std::unique_ptr<QHBoxLayout> main_layout;
    std::unique_ptr<QTabWidget> tab_widget;
    std::unique_ptr<QTreeView>  file_tree_view;

    ExplorerWidgetPrivate() {
        this->main_layout = std::make_unique<QHBoxLayout>();
        this->tab_widget = std::make_unique<QTabWidget>();
        this->file_tree_view = std::make_unique<QTreeView>();
    }
};

ExplorerWidget::ExplorerWidget(QWidget *parent)
    : QWidget{parent},
      _p{new ExplorerWidgetPrivate} {

    auto main_layout = _p->main_layout.get();
    this->setLayout(main_layout);
    main_layout->addWidget(_p->tab_widget.get(), 1);
}

ExplorerWidget::~ExplorerWidget() {
    if(_p) delete _p;
}