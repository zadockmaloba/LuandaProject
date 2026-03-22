#ifndef EXPLORERWIDGET_H
#define EXPLORERWIDGET_H

#include <QWidget>

class ExplorerWidget : public QWidget
{
    Q_OBJECT
    struct ExplorerWidgetPrivate;
    ExplorerWidgetPrivate *_p = nullptr;

public:
    explicit ExplorerWidget(QWidget *parent = nullptr);
    virtual ~ExplorerWidget();

signals:
};

#endif // EXPLORERWIDGET_H
