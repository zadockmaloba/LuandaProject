#include "explorerwidget.h"
#include <QTabWidget>
#include <QTreeWidget>
#include <QTreeView>
#include <QHBoxLayout>
#include <QFileSystemModel>
#include <memory>

struct ExplorerWidget::ExplorerWidgetPrivate {
    std::unique_ptr<QHBoxLayout> main_layout;
    std::unique_ptr<QTabWidget> tab_widget;
    std::unique_ptr<QTreeView>  file_tree_view;
    std::unique_ptr<QFileSystemModel> file_tree_model;

    ExplorerWidgetPrivate() {
        this->main_layout = std::make_unique<QHBoxLayout>();
        this->tab_widget = std::make_unique<QTabWidget>();
        this->file_tree_view = std::make_unique<QTreeView>();
        this->file_tree_model = std::make_unique<QFileSystemModel>();
    }
};

ExplorerWidget::ExplorerWidget(QWidget *parent)
    : QWidget{parent},
      _p{new ExplorerWidgetPrivate} {

    auto main_layout = _p->main_layout.get();
    auto tab_widget = _p->tab_widget.get();
    auto file_tree_view = _p->file_tree_view.get();
    auto file_tree_model = _p->file_tree_model.get();

    file_tree_view->setModel(file_tree_model);

    this->setLayout(main_layout);
    main_layout->addWidget(tab_widget, 1);
    tab_widget->addTab(file_tree_view, "Files");
}

ExplorerWidget::~ExplorerWidget() {
    if(_p) delete _p;
}